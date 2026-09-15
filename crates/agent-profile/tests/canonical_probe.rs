//! THROWAWAY PROBE for the SP3 design §5.4 (never merged). It measures how `std::fs::canonicalize` spells one
//! directory reached through different letter cases and through a symlink, on every CI runner. The test always
//! fails on purpose so the full report is printed in the CI log of every OS.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

fn show(report: &mut String, label: &str, path: &Path) {
    match fs::canonicalize(path) {
        Ok(canonical) => {
            let _ = writeln!(
                report,
                "{label:<34} {} -> {} bytes={:?}",
                path.display(),
                canonical.display(),
                canonical.as_os_str().as_encoded_bytes()
            );
        }
        Err(error) => {
            let _ = writeln!(report, "{label:<34} {} -> ERROR {error}", path.display());
        }
    }
}

#[test]
fn canonicalize_probe_report() {
    let dir = tempfile::tempdir().unwrap();
    let mut report = format!("OS={} tempdir={}\n", std::env::consts::OS, dir.path().display());
    show(&mut report, "tempdir", dir.path());

    let real = dir.path().join("Acme");
    fs::create_dir_all(real.join("Sub")).unwrap();
    let spellings = [
        ("as created (Acme)", dir.path().join("Acme")),
        ("lower (acme)", dir.path().join("acme")),
        ("upper (ACME)", dir.path().join("ACME")),
        ("nested as created (Acme/Sub)", dir.path().join("Acme").join("Sub")),
        ("nested mixed (aCmE/sUB)", dir.path().join("aCmE").join("sUB")),
    ];
    for (label, path) in &spellings {
        show(&mut report, label, path);
    }

    let reference = fs::canonicalize(&real).unwrap();
    let converge = spellings[..3]
        .iter()
        .filter_map(|(_, path)| fs::canonicalize(path).ok())
        .all(|canonical| canonical == reference);
    let _ = writeln!(report, "case spellings that resolve converge on identical bytes: {converge}");

    #[cfg(unix)]
    {
        let link = dir.path().join("link-to-acme");
        std::os::unix::fs::symlink(&real, &link).unwrap();
        show(&mut report, "symlink (link-to-acme)", &link);
        show(&mut report, "symlink child (link-to-acme/Sub)", &link.join("Sub"));
        let lower_link = dir.path().join("link-lower");
        std::os::unix::fs::symlink(dir.path().join("acme"), &lower_link).unwrap();
        show(&mut report, "symlink to lower spelling", &lower_link);
    }
    #[cfg(windows)]
    {
        let junction = dir.path().join("junction-to-acme");
        let status = std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(&junction)
            .arg(&real)
            .output()
            .unwrap();
        let _ = writeln!(report, "mklink /J status={:?}", status.status.code());
        show(&mut report, "junction (junction-to-acme)", &junction);
    }

    panic!("PROBE REPORT (intentional failure)\n{report}");
}
