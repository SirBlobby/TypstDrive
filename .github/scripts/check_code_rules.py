import re
import subprocess
import sys

SLASH_COMMENT = re.compile(r"^\s*(//|/\*)")
MARKUP_COMMENT = re.compile(r"^\s*<!--")
HASH_COMMENT = re.compile(r"^\s*#(?!!)")
ALLOWED_COMMENT = re.compile(r"svelte-ignore|eslint-disable|@ts-(ignore|expect-error|nocheck)")
EMOJI = re.compile(
    "[\U0001F000-\U0001FAFF\U00002600-\U000027BF\U0001F1E6-\U0001F1FF\U00002B00-\U00002BFF️]"
)

COMMENT_STYLES = {
    ".rs": [SLASH_COMMENT],
    ".ts": [SLASH_COMMENT],
    ".js": [SLASH_COMMENT],
    ".css": [SLASH_COMMENT],
    ".svelte": [SLASH_COMMENT, MARKUP_COMMENT],
    ".html": [MARKUP_COMMENT],
    ".yml": [HASH_COMMENT],
    ".yaml": [HASH_COMMENT],
    ".toml": [HASH_COMMENT],
    ".py": [HASH_COMMENT],
    ".sh": [HASH_COMMENT],
    "Dockerfile": [HASH_COMMENT],
}


def comment_styles(path):
    for ending, styles in COMMENT_STYLES.items():
        if path.endswith(ending):
            return styles
    return []


def changed_lines(base_ref):
    diff = subprocess.run(
        ["git", "diff", "--unified=0", "--no-color", f"origin/{base_ref}...HEAD"],
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    path = None
    line_number = 0
    for line in diff.splitlines():
        if line.startswith("+++ "):
            path = line[6:] if line.startswith("+++ b/") else None
        elif line.startswith("@@"):
            line_number = int(re.search(r"\+(\d+)", line).group(1))
        elif line.startswith("+") and path:
            yield path, line_number, line[1:]
            line_number += 1


def problems(base_ref):
    for path, line_number, text in changed_lines(base_ref):
        if EMOJI.search(text):
            yield f"{path}:{line_number}: emoji found"
        is_comment = any(style.match(text) for style in comment_styles(path))
        if is_comment and not ALLOWED_COMMENT.search(text):
            yield f"{path}:{line_number}: code comment found"


def main():
    found = list(problems(sys.argv[1]))
    for problem in found:
        print(problem)
    if found:
        print(f"{len(found)} problem(s). Code comments and emojis are not allowed.")
        sys.exit(1)
    print("No code comments or emojis added.")


if __name__ == "__main__":
    main()
