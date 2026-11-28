import re
import subprocess
import sys

EMOJI = re.compile(
    "[\U0001F000-\U0001FAFF\U00002600-\U000027BF\U0001F1E6-\U0001F1FF\U00002B00-\U00002BFF️]"
)


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


def main():
    found = list(problems(sys.argv[1]))
    for problem in found:
        print(problem)
    if found:
        print(f"{len(found)} problem(s). Emojis are not allowed in code or documentation.")
        sys.exit(1)
    print("No emojis added.")


if __name__ == "__main__":
    main()
