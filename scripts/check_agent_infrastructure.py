#!/usr/bin/env python3
"""Validate DroidOps agent documents without dependencies or external commands."""

import argparse
from pathlib import Path
import re
import tempfile
import unittest
from urllib.parse import unquote, urlsplit


SKILLS = (
    "rust-architecture", "egui-development", "adb-fastboot",
    "migration-tauri-to-egui", "git-workflow", "rust-testing",
    "release-management", "github-actions", "documentation",
    "security-review", "performance-optimization", "feature-development",
)
SECTIONS = (
    "Purpose", "When to use", "Required context", "Implementation workflow",
    "Coding standards", "Safety constraints", "Validation commands",
    "Definition of done", "Relevant documentation",
)
CANONICAL_SECTIONS = (
    "Project identity", "Architecture rules", "Rust coding standards",
    "egui coding standards", "Device and host safety", "Agent workflow",
    "Git and release boundaries", "Validation available today",
)


def prose(text):
    """Remove fenced/inline code so examples are not interpreted as guidance."""
    result = []
    fence = None
    for line in text.splitlines():
        marker = re.match(r"^ {0,3}(`{3,}|~{3,})(.*)$", line)
        if marker:
            token, suffix = marker.groups()
            if fence is None:
                fence = token
            elif token[0] == fence[0] and len(token) >= len(fence) and not suffix.strip():
                fence = None
            result.append("")
        else:
            result.append(line if fence is None else "")
    return re.sub(r"(`+)(.*?)\1", "", "\n".join(result))


def section_errors(text, sections):
    """Require a nonempty level-two section for each project contract item."""
    found = dict(re.findall(r"^## ([^\n]+)\n(.*?)(?=^#{1,6} |\Z)", prose(text), re.M | re.S))
    return [f"missing or empty section: {name}" for name in sections if not found.get(name, "").strip()]


def metadata_errors(text, directory):
    """Validate the documented two-field plain-string YAML subset, not general YAML."""
    lines = text.splitlines()
    if not lines or lines[0] != "---" or "---" not in lines[1:]:
        return ["missing or unclosed frontmatter"]
    end = lines.index("---", 1)
    fields = {}
    errors = []
    for line in lines[1:end]:
        match = re.fullmatch(r"([a-z][a-z-]*):(?: (.*))?", line)
        if not match:
            errors.append("unsupported metadata syntax; use single-line plain strings")
            continue
        key, value = match.groups()
        value = (value or "").strip()
        if key not in ("name", "description"):
            errors.append(f"unsupported metadata field: {key}")
        if key in fields:
            errors.append(f"duplicate metadata field: {key}")
        fields[key] = value
        if (not re.match(r"^[A-Za-z]", value) or ": " in value
                or any(char in value for char in "#[]{}<>")
                or value.lower() in ("true", "false", "null", "yes", "no", "on", "off")):
            errors.append(f"invalid metadata value for {key}; use a plain nonempty string")
    name = fields.get("name", "")
    if len(name) > 64 or not re.fullmatch(r"[a-z0-9]+(?:-[a-z0-9]+)*", name):
        errors.append("invalid name; use at most 64 lowercase letters, digits, and single hyphens")
    if name != directory:
        errors.append("name must match parent directory")
    description = fields.get("description", "")
    if not description or len(description) > 1024:
        errors.append("description must contain 1 to 1024 characters")
    return errors


def local_links(text, source, root):
    """Return resolved local targets and errors for inline and explicit reference links."""
    content = prose(text)
    targets = []
    errors = []
    definitions = {
        label.strip().casefold(): target for label, target in re.findall(
            r"^ {0,3}\[([^\]\n]+)\]:\s*(<[^>\n]+>|[^\s]+)", content, re.M
        )
    }
    raw_targets = re.findall(
        r"!?\[[^\]\n]*\]\(\s*(<[^>\n]+>|[^\s)\n]+)(?:\s+[\"'][^\n]*?[\"'])?\s*\)",
        content,
    )
    for label in re.findall(r"\[[^\]\n]*\]\[([^\]\n]+)\]", content):
        key = label.strip().casefold()
        if key not in definitions:
            errors.append(f"undefined reference: {label}")
        else:
            raw_targets.append(definitions[key])
    for raw in raw_targets:
        target = raw[1:-1] if raw.startswith("<") else raw
        try:
            url = urlsplit(target)
            if url.scheme in ("https", "http", "mailto"):
                continue
            path = Path(unquote(url.path))
            if url.scheme or url.netloc or path.is_absolute() or "\\" in str(path):
                raise ValueError("outside or unsupported link target")
            if not url.path:
                continue  # Heading fragments are intentionally not validated.
            resolved = (source.parent / path).resolve()
            resolved.relative_to(root)
        except (ValueError, OSError, RuntimeError):
            errors.append(f"outside or unsupported link target: {target}")
            continue
        targets.append(resolved)
        if not resolved.is_file():
            errors.append(f"missing link target: {target}")
    return targets, errors


def validate(root):
    """Return actionable validation errors for the infrastructure at root."""
    root = root.resolve()
    files = [
        "AGENTS.md", "CLAUDE.md", ".github/copilot-instructions.md",
        "docs/development/AI_AGENTS.md",
    ] + [f".agents/skills/{name}/SKILL.md" for name in SKILLS]
    errors = []
    for name in files:
        path = root / name
        if not path.is_file():
            errors.append(f"{name}: missing file")
            continue
        try:
            path.resolve().relative_to(root)
            text = path.read_text(encoding="utf-8")
        except (OSError, UnicodeError, ValueError, RuntimeError) as exc:
            errors.append(f"{name}: cannot read file within repository: {exc}")
            continue
        issues = []
        if name.endswith("/SKILL.md"):
            issues.extend(metadata_errors(text, path.parent.name))
            issues.extend(section_errors(text, SECTIONS))
        elif name == "AGENTS.md":
            issues.extend(section_errors(text, CANONICAL_SECTIONS))
        links, link_errors = local_links(text, path, root)
        issues.extend(link_errors)
        if name in ("CLAUDE.md", ".github/copilot-instructions.md") and root / "AGENTS.md" not in links:
            issues.append("adapter must contain a canonical link to AGENTS.md")
        errors.extend(f"{name}: {issue}" for issue in issues)
    return errors


class InfrastructureTests(unittest.TestCase):
    """Exercise acceptance and rejection against actual temporary file trees."""

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="droidops-agent-check-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        files = {
            "AGENTS.md": "# Policy\n" + "".join(
                "\n## " + section + "\nProject guidance.\n" for section in (
                    "Project identity", "Architecture rules", "Rust coding standards",
                    "egui coding standards", "Device and host safety", "Agent workflow",
                    "Git and release boundaries", "Validation available today",
                )
            ),
            "CLAUDE.md": "Read [canonical](AGENTS.md).\n",
            ".github/copilot-instructions.md": "Read [canonical](../AGENTS.md).\n",
            "docs/development/AI_AGENTS.md": "Read [canonical](../../AGENTS.md).\n",
        }
        for name in SKILLS:
            files[f".agents/skills/{name}/SKILL.md"] = (
                f"---\nname: {name}\ndescription: Review device work when requested.\n---\n"
                + "".join(f"\n## {section}\nRead the project guidance.\n" for section in SECTIONS)
                + "\n[Canonical](../../../AGENTS.md)\n"
            )
        for name, content in files.items():
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")
        self.skill = self.root / ".agents/skills/adb-fastboot/SKILL.md"

    def change(self, old, new):
        self.skill.write_text(
            self.skill.read_text(encoding="utf-8").replace(old, new), encoding="utf-8"
        )

    def rejects(self, phrase):
        errors = validate(self.root)
        self.assertTrue(errors, "invalid infrastructure must fail")
        self.assertTrue(any(phrase in error for error in errors), errors)

    def test_complete_tree_accepts_crlf_external_links_and_fenced_examples(self):
        with self.skill.open("a", encoding="utf-8") as handle:
            handle.write(
                "\n[spec](https://agentskills.io/specification)\n"
                "[contact](mailto:maintainer@example.invalid)\n"
                "[section](#purpose)\n```md\n[example](missing.md)\n```\n"
                "~~~md\n[example](missing-too.md)\n~~~\n"
                "`[inline example](missing-inline.md)`\n"
            )
        self.skill.write_bytes(self.skill.read_bytes().replace(b"\n", b"\r\n"))
        self.assertEqual(validate(self.root), [])

    def test_missing_required_file_is_reported(self):
        self.skill.unlink()
        self.rejects("missing file")

    def test_metadata_rejections(self):
        original = self.skill.read_text(encoding="utf-8")
        mutations = (
            ("name: adb-fastboot", "name: another-name", "directory"),
            ("name: adb-fastboot", "name: adb-fastboot\nname: adb-fastboot", "duplicate"),
            ("name: adb-fastboot", "name: adb--fastboot", "name"),
            ("name: adb-fastboot", "name: " + "a" * 65, "name"),
            ("description: Review device work when requested.", "description: ", "description"),
            ("description: Review device work when requested.", "description: " + "x" * 1025, "description"),
            ("description: Review device work when requested.", "description: [not, a, string]", "metadata"),
            ("description: Review device work when requested.", "description: true", "metadata"),
            ("description: Review device work when requested.", "description: misleading # comment", "metadata"),
            ("description: Review device work when requested.", "description: value: nested", "metadata"),
            ("description: Review device work when requested.", "description: |\n  multiline", "metadata"),
            ("name: adb-fastboot", "name: adb-fastboot\nallowed-tools: shell", "metadata"),
            ("---\nname:", "name:", "frontmatter"),
        )
        for old, new, phrase in mutations:
            with self.subTest(value=new[:70]):
                self.skill.write_text(original.replace(old, new), encoding="utf-8")
                self.rejects(phrase)

    def test_empty_required_section_is_rejected(self):
        self.change("## Safety constraints\nRead the project guidance.", "## Safety constraints\n")
        self.rejects("Safety constraints")

    def test_missing_canonical_section_is_rejected(self):
        path = self.root / "AGENTS.md"
        path.write_text("# Project\n", encoding="utf-8")
        self.rejects("Project identity")

    def test_adapter_must_reference_canonical_file(self):
        (self.root / "CLAUDE.md").write_text("AGENTS.md is mentioned only.\n", encoding="utf-8")
        self.rejects("canonical link")

    def test_broken_relative_link_is_rejected(self):
        with self.skill.open("a", encoding="utf-8") as handle:
            handle.write("\n[missing](../../../missing.md)\n")
        self.rejects("missing link target")

    def test_reference_links_are_checked(self):
        with self.skill.open("a", encoding="utf-8") as handle:
            handle.write("\n[project][policy]\n[policy]: ../../../AGENTS.md\n")
        self.assertEqual(validate(self.root), [])
        self.change("[policy]: ../../../AGENTS.md", "[policy]: ../../../absent.md")
        self.rejects("missing link target")

    def test_undefined_reference_is_rejected(self):
        with self.skill.open("a", encoding="utf-8") as handle:
            handle.write("\n[project][undefined]\n")
        self.rejects("undefined reference")

    def test_escaping_encoded_and_absolute_paths_are_rejected(self):
        original = self.skill.read_text(encoding="utf-8")
        for target in ("../../../../outside.md", "%2Fetc%2Fpasswd", "file:///etc/passwd"):
            with self.subTest(target=target):
                self.skill.write_text(original + f"\n[escape]({target})\n", encoding="utf-8")
                self.rejects("outside or unsupported")

    def test_links_with_spaces_and_fragments(self):
        (self.root / "a guide.md").write_text("# Guide\n", encoding="utf-8")
        with self.skill.open("a", encoding="utf-8") as handle:
            handle.write("\n[guide](<../../../a guide.md#intro>)\n")
        self.assertEqual(validate(self.root), [])

    def test_invalid_utf8_reports_failure(self):
        self.skill.write_bytes(b"\xff")
        self.rejects("cannot read")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true", help="run temporary-fixture tests only")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    if args.self_test:
        result = unittest.TextTestRunner(verbosity=2).run(
            unittest.defaultTestLoader.loadTestsFromTestCase(InfrastructureTests)
        )
        return 0 if result.wasSuccessful() else 1
    errors = validate(args.root)
    for error in errors:
        print(f"ERROR: {error}")
    if errors:
        print(f"Agent infrastructure validation failed: {len(errors)} error(s).")
        return 1
    print("Agent infrastructure valid: 12 skills and 4 instruction/index documents.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
