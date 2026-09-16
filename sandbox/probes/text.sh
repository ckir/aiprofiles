# Text handling shared by the probe (sandbox/probes/common.sh) and the transcript assembler
# (sandbox/transcript.sh). Sourcing this file has no side effects, which is why it is separate from
# common.sh: common.sh installs an EXIT trap and truncates a file the moment it is sourced, so the
# assembler cannot source it.
#
# SP4 design §7.4: "Captured output is untrusted vendor text. Before it becomes a committed file it is
# stripped of terminal control sequences and bounded in size; an excerpt that must be truncated says so at
# the truncation point." Both halves live here, in one copy, because they have two callers.

# probe_strip_ansi: remove terminal control sequences and CR line endings from stdin.
#
# The version extraction needs this as much as the transcript does. `ESC[0m` contains the run `0m`, so a
# vendor that colours its `--version` banner would otherwise hand the registry a version of `0m` — and
# Gate A accepts it, because `0m` is inside the charset it checks. `\033` is not portable sed syntax, so
# the escape is built by printf and interpolated.
#
# Two passes, and the second is the one that makes this safe rather than tidy. The first removes CSI
# sequences, which is what produces spurious digit runs. The second deletes every remaining control
# character except tab and newline, so no escape byte reaches a committed file however it was spelled —
# an OSC title, a lone BEL, a NUL from a binary help text. Enumerating each sequence family instead would
# mean the file is safe only against the families someone thought of.
# The `s` command uses a comma delimiter because the CSI intermediate range `[ -/]` ends at a slash, and
# a slash inside a bracket expression is not reliably exempt from ending an `s/.../.../`.
probe_strip_ansi() {
    probe_esc=$(printf '\033')
    sed "s,${probe_esc}\\[[0-9;?]*[ -/]*[@-~],,g" | tr -d '\000-\010\013-\037\177'
}

# probe_excerpt <max-lines>: strip control sequences, bound the text, and say where it was cut.
#
# Both bounds are stated rather than silent. A transcript that stops mid-way with no marker reads as a
# complete record of a short output, and §5.3 commits it byte-for-byte, so an unmarked truncation is a
# durable false statement about what the agent printed.
probe_excerpt() {
    probe_strip_ansi | awk -v max="$1" '
        NR <= max {
            if (length($0) > 500) {
                print substr($0, 1, 500) "  [line truncated]"
            } else {
                print
            }
        }
        NR == max + 1 { print "  [truncated: more than " max " lines]" }
    '
}
