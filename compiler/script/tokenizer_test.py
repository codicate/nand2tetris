import subprocess
import os

# === CONFIG: List of directories ===
DIRS = [
    "ArrayTest",
    "ExpressionLessSquare",
    "Square",
]

# === Config: Diff command ===
DIFF_CMD = "diff"

for dir_name in DIRS:
    print(dir_name + ":")
    # Run cargo run -- <dir_name>
    cargo_cmd = ["cargo", "run", "--", dir_name]
    try:
        subprocess.run(cargo_cmd, check=True, capture_output=True)
    except subprocess.CalledProcessError:
        print(f"❌ cargo run failed in {dir_name}")
        continue

    # Change into directory
    try:
        os.chdir(dir_name)
    except FileNotFoundError:
        print(f"❌ Failed to enter directory {dir_name}")
        continue

    # List all *.tokens.xml files
    tokens_files = [f for f in os.listdir(".") if f.endswith(".tokens.xml")]

    for tokens_file in tokens_files:
        base = tokens_file[:-len(".tokens.xml")]
        target_file = f"{base}T.xml"

        if os.path.isfile(target_file):
            # Run diff -q --strip-trailing-cr target_file tokens_file
            diff_cmd = [DIFF_CMD, "-q", "--strip-trailing-cr", target_file, tokens_file]

            result = subprocess.run(diff_cmd, capture_output=True)
            if result.returncode == 0:
                print(f"✅ {tokens_file} and {target_file} are the same")
            else:
                print(f"❌ {tokens_file} and {target_file} differ")
        else:
            print(f"⚠️  Missing file: {target_file}")

    # Go back to parent directory of dir_name
    os.chdir("..")
