import os
import sys
import subprocess
import shutil

# Configuration
REPO_URL = "https://github.com/pku-minic/compiler-dev-test-cases"
REPO_DIR = "compiler-dev-test-cases"
TEST_DIRS = ["lv1", "lv2", "lv3", "lv4", "lv5", "lv6", "lv7", "lv8", "lv9", "perf"]
KIRA_BIN = "target/release/kira"
GCC_RISCV = "riscv64-linux-gnu-gcc"
QEMU_RISCV = "qemu-riscv64-static"
SYLIB_SRC = "scripts/sylib.c"
SYLIB_HEADER = "scripts/sylib.h"
SYLIB_OBJ = "scripts/sylib.o"

def log(msg):
    print(f"[TEST] {msg}")

def run_cmd(cmd, check=True, input=None, timeout=None):
    try:
        result = subprocess.run(
            cmd,
            input=input,
            capture_output=True,
            text=True, # assumes utf-8
            check=check,
            timeout=timeout
        )
        return result
    except subprocess.CalledProcessError as e:
        if not check:
            return e
        log(f"Command failed: {' '.join(cmd)}")
        log(f"Stderr: {e.stderr}")
        raise e

def main():
    # 1. Clone repo
    if not os.path.exists(REPO_DIR):
        log(f"Cloning {REPO_URL}...")
        subprocess.run(["git", "clone", REPO_URL, REPO_DIR], check=True)
    else:
        log("Test repo already exists.")

    # 2. Build sylib
    log("Building sylib...")
    run_cmd([GCC_RISCV, "-c", SYLIB_SRC, "-o", SYLIB_OBJ])

    # 3. Check Kira binary
    if not os.path.exists(KIRA_BIN):
        log(f"Kira binary not found at {KIRA_BIN}. Building...")
        subprocess.run(["cargo", "build", "--release"], check=True)
    
    total = 0
    passed = 0
    failed = []

    # 4. Run tests
    for d in TEST_DIRS:
        dir_path = os.path.join(REPO_DIR, "testcases", d)
        if not os.path.exists(dir_path):
            log(f"Directory {dir_path} does not exist, skipping.")
            continue
        
        log(f"Running tests in {d}...")
        files = sorted([f for f in os.listdir(dir_path) if f.endswith(".sy") or f.endswith(".c")])
        
        for f in files:
            total += 1
            full_path = os.path.join(dir_path, f)
            base_name = os.path.splitext(f)[0]
            input_file = os.path.join(dir_path, base_name + ".in")
            
            # Read input if exists
            test_input = ""
            if os.path.exists(input_file):
                with open(input_file, "r") as inf:
                    test_input = inf.read()
            
            log(f"Testing {f}...")
            
            # --- Compile with Kira ---
            out_asm = "output.S"
            out_bin = "output"
            try:
                # Kira compile to ASM
                run_cmd([KIRA_BIN, "-riscv", full_path, "-o", out_asm])
                
                # GCC assemble and link
                run_cmd([GCC_RISCV, out_asm, SYLIB_OBJ, "-o", out_bin, "-static"])
                
                # Run with QEMU
                res_kira = run_cmd([QEMU_RISCV, out_bin], check=False, input=test_input, timeout=10)
                
            except Exception as e:
                log(f"FAIL: Kira failed on {f}: {e}")
                failed.append(f"{d}/{f} (compile/run failed)")
                continue

            # --- Compile with GCC (Reference) ---
            ref_bin = "reference"
            try:
                # GCC compile source + sylib
                # Need to include header path
                cmd = [GCC_RISCV, full_path, SYLIB_OBJ, "-o", ref_bin, "-static", "-Iscripts", "-include", "sylib.h"]
                # For .sy files, treat as C? SysY is mostly C but might have some diffs.
                # Usually .sy is compatible with C if header is included.
                # But if .sy file extension is used, gcc might not like it.
                # Rename temporarily or use -x c
                run_cmd(cmd + ["-x", "c"]) 
                
                # Run with QEMU
                res_ref = run_cmd([QEMU_RISCV, ref_bin], check=False, input=test_input, timeout=10)
                
            except Exception as e:
                log(f"FAIL: Reference GCC failed on {f}: {e}")
                failed.append(f"{d}/{f} (reference failed)")
                continue
            
            # --- Compare ---
            # Compare exit code
            if res_kira.returncode != res_ref.returncode:
                log(f"FAIL: Exit code mismatch for {f}. Kira: {res_kira.returncode}, Ref: {res_ref.returncode}")
                failed.append(f"{d}/{f} (exit code)")
                continue
            
            # Compare stdout (strip whitespace)
            out_kira = res_kira.stdout.strip()
            out_ref = res_ref.stdout.strip()
            
            # Sometimes output might differ slightly in whitespace or due to timing logs
            # If starttime/stoptime prints output, it should be deterministic if mocked properly
            # In sylib.c, I mocked it to use gettimeofday, which is NOT deterministic.
            # But the output format is "Timer@...".
            # If the test relies on specific timing values, it will fail.
            # But usually tests check logic.
            # If output is different, check if it's just timing.
            
            if out_kira != out_ref:
                # filter out Timer lines?
                lines_kira = [l for l in out_kira.splitlines() if not l.startswith("Timer@")]
                lines_ref = [l for l in out_ref.splitlines() if not l.startswith("Timer@")]
                
                if lines_kira != lines_ref:
                    log(f"FAIL: Output mismatch for {f}")
                    log(f"Kira output:\n{out_kira}")
                    log(f"Ref output:\n{out_ref}")
                    failed.append(f"{d}/{f} (output)")
                    continue

            log(f"PASS: {f}")
            passed += 1

    log("="*30)
    log(f"Total: {total}, Passed: {passed}, Failed: {len(failed)}")
    if failed:
        log("Failed tests:")
        for f in failed:
            log(f"  {f}")
        sys.exit(1)
    else:
        log("All tests passed!")

if __name__ == "__main__":
    main()
