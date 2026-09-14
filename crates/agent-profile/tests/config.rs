//! Configuration reads and locked atomic writes through the public API (SP1 design §8.3; spec §34
//! "Configuration"). The failed-replacement and reader-during-writes cases that need the crate-private
//! `update_with` hook are unit tests inside `config`.

use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use agent_profile::config::{self, AppRoot, Config};
use agent_profile::error::Error;

fn root() -> (tempfile::TempDir, AppRoot) {
    let dir = tempfile::tempdir().unwrap();
    let root = AppRoot::from_path(dir.path().to_path_buf());
    (dir, root)
}

fn absolute(name: &str) -> String {
    std::env::temp_dir().join(name).to_str().unwrap().to_owned()
}

fn set_agent(doc: &mut toml_edit::DocumentMut, id: &str, executable: &str) {
    let agents = doc
        .entry("agents")
        .or_insert_with(|| {
            let mut agents = toml_edit::Table::new();
            agents.set_implicit(true);
            toml_edit::Item::Table(agents)
        })
        .as_table_mut()
        .expect("agents is a table");
    let mut agent = toml_edit::Table::new();
    agent.insert("executable", toml_edit::value(executable));
    agents.insert(id, toml_edit::Item::Table(agent));
}

#[test]
fn valid_toml_reads_the_configured_executable() {
    let (_dir, root) = root();
    let exe = absolute("fake-agent");
    std::fs::write(root.config_path(), format!("[agents.fake]\nexecutable = {exe:?}\n")).unwrap();
    let config = Config::load(&root).unwrap();
    assert_eq!(config.agent_executable("fake"), Some(Path::new(&exe)));
}

#[test]
fn missing_file_is_an_empty_configuration() {
    let (_dir, root) = root();
    assert_eq!(Config::load(&root).unwrap(), Config::default());
}

#[test]
fn invalid_configurations_are_errors() {
    let (_dir, root) = root();
    for text in [
        "[agents\n",
        "[agents.fake]\nexecutable = 3\n",
        "[agents.fake]\nunknown = \"x\"\n",
        "[agents.fake]\nexecutable = \"relative/path\"\n",
    ] {
        std::fs::write(root.config_path(), text).unwrap();
        assert!(matches!(Config::load(&root), Err(Error::ConfigInvalid { .. })), "{text:?}");
    }
}

#[test]
fn update_refuses_a_corrupt_file_and_leaves_it_byte_identical() {
    let (_dir, root) = root();
    let corrupt = b"[agents.fake\n\xffexecutable".to_vec();
    std::fs::write(root.config_path(), &corrupt).unwrap();
    let error = config::update(&root, |doc| {
        set_agent(doc, "fake", &absolute("x"));
        Ok(())
    })
    .unwrap_err();
    assert!(matches!(error, Error::ConfigInvalid { .. }), "{error:?}");
    assert_eq!(std::fs::read(root.config_path()).unwrap(), corrupt);
}

#[test]
fn update_preserves_comments_and_key_order() {
    let (_dir, root) = root();
    let original = format!(
        "# my agents\n[agents.zeta]\nexecutable = {:?} # trailing\n\n[agents.alpha]\n",
        absolute("zeta")
    );
    std::fs::write(root.config_path(), &original).unwrap();
    config::update(&root, |doc| {
        set_agent(doc, "fake", &absolute("fake"));
        Ok(())
    })
    .unwrap();
    let updated = std::fs::read_to_string(root.config_path()).unwrap();
    assert!(updated.starts_with(&original), "{updated}");
    assert!(updated.contains("# trailing"), "{updated}");
    let config = Config::load(&root).unwrap();
    assert_eq!(config.configured_agents().collect::<Vec<_>>(), ["alpha", "fake", "zeta"]);
}

#[test]
fn concurrent_writers_lose_no_update() {
    let (_dir, root) = root();
    let writers: Vec<_> = (0..8)
        .map(|index| {
            let root = root.clone();
            thread::spawn(move || {
                config::update(&root, |doc| {
                    set_agent(doc, &format!("agent{index}"), &absolute(&format!("a{index}")));
                    Ok(())
                })
                .unwrap();
            })
        })
        .collect();
    for writer in writers {
        writer.join().unwrap();
    }
    let config = Config::load(&root).unwrap();
    assert_eq!(config.configured_agents().count(), 8);
    for index in 0..8 {
        assert_eq!(
            config.agent_executable(&format!("agent{index}")),
            Some(Path::new(&absolute(&format!("a{index}")))),
            "agent{index}"
        );
    }
}

#[test]
fn a_waiting_writer_edits_the_first_writers_result() {
    let (_dir, root) = root();
    let (locked_tx, locked_rx) = mpsc::channel();
    let first = {
        let root = root.clone();
        thread::spawn(move || {
            config::update(&root, |doc| {
                set_agent(doc, "first", &absolute("first"));
                locked_tx.send(()).unwrap();
                thread::sleep(Duration::from_millis(300));
                Ok(())
            })
            .unwrap();
        })
    };
    locked_rx.recv().unwrap();
    config::update(&root, |doc| {
        assert!(doc.get("agents").and_then(|agents| agents.get("first")).is_some(), "{doc}");
        set_agent(doc, "second", &absolute("second"));
        Ok(())
    })
    .unwrap();
    first.join().unwrap();
    let config = Config::load(&root).unwrap();
    assert_eq!(config.configured_agents().collect::<Vec<_>>(), ["first", "second"]);
}

#[test]
fn update_times_out_while_another_handle_holds_the_lock() {
    let (_dir, root) = root();
    let previous = format!("[agents.fake]\nexecutable = {:?}\n", absolute("old"));
    std::fs::write(root.config_path(), &previous).unwrap();
    let holder = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(root.path().join("config.toml.lock"))
        .unwrap();
    holder.lock().unwrap();

    // A watchdog, so a regression that retries forever fails instead of hanging the suite.
    let (done_tx, done_rx) = mpsc::channel();
    let writer_root = root.clone();
    let started = Instant::now();
    thread::spawn(move || {
        let result = config::update(&writer_root, |doc| {
            set_agent(doc, "fake", &absolute("new"));
            Ok(())
        });
        let _ = done_tx.send(result);
    });
    let result =
        done_rx.recv_timeout(Duration::from_secs(30)).expect("update did not give up within 30 s");
    let elapsed = started.elapsed();

    match result {
        Err(Error::ConfigWrite { source, .. }) => assert!(
            source
                .to_string()
                .contains("another agent-profile process holds the configuration lock"),
            "{source}"
        ),
        other => panic!("{other:?}"),
    }
    assert!(elapsed >= Duration::from_secs(9), "gave up after {elapsed:?}, before the 10 s bound");
    assert_eq!(std::fs::read_to_string(root.config_path()).unwrap(), previous);
    drop(holder);
}

#[test]
fn an_edit_that_breaks_the_schema_is_refused() {
    let (_dir, root) = root();
    let error = config::update(&root, |doc| {
        doc["default"] = toml_edit::value("work");
        Ok(())
    })
    .unwrap_err();
    assert!(matches!(error, Error::ConfigInvalid { .. }), "{error:?}");
    assert!(!root.config_path().exists());
}

#[cfg(windows)]
#[test]
fn windows_replace_blocked_by_an_open_handle_preserves_the_previous_file() {
    use std::os::windows::fs::OpenOptionsExt;
    use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ;

    let (_dir, root) = root();
    let previous = format!("[agents.fake]\nexecutable = {:?}\n", absolute("old"));
    std::fs::write(root.config_path(), &previous).unwrap();
    let _holder = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .open(root.config_path())
        .unwrap();
    let error = config::update(&root, |doc| {
        set_agent(doc, "fake", &absolute("new"));
        Ok(())
    })
    .unwrap_err();
    assert!(matches!(error, Error::ConfigWrite { .. }), "{error:?}");
    assert_eq!(std::fs::read_to_string(root.config_path()).unwrap(), previous);
    let leftovers: Vec<_> = std::fs::read_dir(root.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.starts_with(".config.toml.") && name.ends_with(".tmp"))
        .collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
}
