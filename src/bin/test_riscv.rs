use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const REPO_URL: &str = "https://github.com/pku-minic/compiler-dev-test-cases";
const REPO_DIR: &str = "compiler-dev-test-cases";
const TEST_DIRS: [&str; 10] = [
  "lv1", "lv2", "lv3", "lv4", "lv5", "lv6", "lv7", "lv8", "lv9", "perf",
];
const KIRA_BIN: &str = "target/release/kira";
const GCC_RISCV: &str = "riscv64-linux-gnu-gcc";
const QEMU_RISCV: &str = "qemu-riscv64-static";
const SYLIB_SRC: &str = "scripts/sylib.c";
const SYLIB_OBJ: &str = "scripts/sylib.o";
const SYLIB_HEADER: &str = "sylib.h";

fn main() {
  if let Err(err) = try_main() {
    eprintln!("{err}");
    process::exit(1);
  }
}

fn try_main() -> Result<(), String> {
  let args = Args::parse()?;
  let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

  let repo_dir = workspace.join(REPO_DIR);
  let kira_bin = workspace.join(KIRA_BIN);
  let sylib_src = workspace.join(SYLIB_SRC);
  let sylib_obj = workspace.join(SYLIB_OBJ);

  if !repo_dir.exists() {
    log(&format!("Cloning {}...", REPO_URL));
    run_cmd(
      vec![
        "git".to_string(),
        "clone".to_string(),
        REPO_URL.to_string(),
        repo_dir.to_string_lossy().to_string(),
      ],
      true,
      None,
      None,
      Some(&workspace),
    )?;
  } else {
    log("Test repo already exists.");
  }

  log("Building sylib...");
  run_cmd(
    vec![
      GCC_RISCV.to_string(),
      "-c".to_string(),
      sylib_src.to_string_lossy().to_string(),
      "-o".to_string(),
      sylib_obj.to_string_lossy().to_string(),
    ],
    true,
    None,
    None,
    Some(&workspace),
  )?;

  if !kira_bin.exists() {
    log(&format!(
      "Kira binary not found at {}. Building...",
      kira_bin.to_string_lossy()
    ));
    run_cmd(
      vec![
        "cargo".to_string(),
        "build".to_string(),
        "--release".to_string(),
      ],
      true,
      None,
      None,
      Some(&workspace),
    )?;
  }

  let mut tests = Vec::new();

  if let Some(input_c_file) = args.input_c_file {
    let input_path = if input_c_file.is_absolute() {
      input_c_file
    } else {
      env::current_dir()
        .map_err(|e| format!("failed to get current directory: {e}"))?
        .join(input_c_file)
    };

    if !input_path.exists() {
      return Err(format!(
        "input c file not found: {}",
        input_path.to_string_lossy()
      ));
    }
    if input_path.extension() != Some(OsStr::new("c")) {
      return Err("--input-c-file must point to a .c file".to_string());
    }

    tests.push(TestCase {
      group: "custom".to_string(),
      source: input_path,
    });
  } else {
    for d in TEST_DIRS {
      let dir_path = repo_dir.join("testcases").join(d);
      if !dir_path.exists() {
        log(&format!(
          "Directory {} does not exist, skipping.",
          dir_path.to_string_lossy()
        ));
        continue;
      }

      log(&format!("Running tests in {d}..."));
      let mut entries = fs::read_dir(&dir_path)
        .map_err(|e| format!("failed to read {}: {e}", dir_path.to_string_lossy()))?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| matches!(p.extension().and_then(OsStr::to_str), Some("sy" | "c")))
        .collect::<Vec<_>>();
      entries.sort();

      tests.extend(entries.into_iter().map(|source| TestCase {
        group: d.to_string(),
        source,
      }));
    }
  }

  let mut passed = 0usize;
  let mut failed = Vec::new();
  let total = tests.len();

  for case in tests {
    let file_name = case
      .source
      .file_name()
      .and_then(OsStr::to_str)
      .unwrap_or("<unknown>");
    log(&format!("Testing {file_name}..."));

    match run_single_test(&workspace, &case, &kira_bin, &sylib_obj) {
      Ok(()) => {
        log(&format!("PASS: {file_name}"));
        passed += 1;
      }
      Err(err) => {
        log(&format!("FAIL: {file_name}: {err}"));
        failed.push(format!("{}/{} ({err})", case.group, file_name));
      }
    }
  }

  log("==============================");
  log(&format!(
    "Total: {total}, Passed: {passed}, Failed: {}",
    failed.len()
  ));

  if !failed.is_empty() {
    log("Failed tests:");
    for f in &failed {
      log(&format!("  {f}"));
    }
    return Err("test run failed".to_string());
  }

  log("All tests passed!");
  Ok(())
}

#[derive(Clone)]
struct TestCase {
  group: String,
  source: PathBuf,
}

struct Args {
  input_c_file: Option<PathBuf>,
}

impl Args {
  fn parse() -> Result<Self, String> {
    let mut args = env::args().skip(1);
    let mut input_c_file = None;

    while let Some(arg) = args.next() {
      match arg.as_str() {
        "--input-c-file" => {
          let value = args
            .next()
            .ok_or_else(|| "missing value for --input-c-file".to_string())?;
          input_c_file = Some(PathBuf::from(value));
        }
        "-h" | "--help" => {
          print_usage();
          process::exit(0);
        }
        other => return Err(format!("unknown argument: {other}")),
      }
    }

    Ok(Self { input_c_file })
  }
}

fn print_usage() {
  println!(
    "Usage: cargo run --bin test_riscv -- [--input-c-file <path/to/test.c>]\n\n\
Options:\n  --input-c-file <path>   Run only the specified .c file instead of all testcases\n  -h, --help              Show this help"
  );
}

fn run_single_test(
  workspace: &Path,
  case: &TestCase,
  kira_bin: &Path,
  sylib_obj: &Path,
) -> Result<(), String> {
  let input_file = case.source.with_extension("in");
  let test_input = if input_file.exists() {
    fs::read_to_string(&input_file)
      .map_err(|e| format!("failed to read input {}: {e}", input_file.to_string_lossy()))?
  } else {
    String::new()
  };

  let out_asm = workspace.join("output.S");
  let out_bin = workspace.join("output");
  let ref_bin = workspace.join("reference");

  run_cmd(
    vec![
      kira_bin.to_string_lossy().to_string(),
      "-riscv".to_string(),
      case.source.to_string_lossy().to_string(),
      "-o".to_string(),
      out_asm.to_string_lossy().to_string(),
    ],
    true,
    None,
    None,
    Some(workspace),
  )
  .map_err(|e| format!("compile/run failed: {e}"))?;

  run_cmd(
    vec![
      GCC_RISCV.to_string(),
      out_asm.to_string_lossy().to_string(),
      sylib_obj.to_string_lossy().to_string(),
      "-o".to_string(),
      out_bin.to_string_lossy().to_string(),
      "-static".to_string(),
    ],
    true,
    None,
    None,
    Some(workspace),
  )
  .map_err(|e| format!("compile/run failed: {e}"))?;

  let kira_result = run_cmd(
    vec![
      QEMU_RISCV.to_string(),
      out_bin.to_string_lossy().to_string(),
    ],
    false,
    Some(&test_input),
    Some(Duration::from_secs(10)),
    Some(workspace),
  )
  .map_err(|e| format!("compile/run failed: {e}"))?;

  let mut ref_cmd = vec![GCC_RISCV.to_string()];
  if case.source.extension() != Some(OsStr::new("c")) {
    ref_cmd.extend(["-x".to_string(), "c".to_string()]);
  }
  ref_cmd.extend([
    case.source.to_string_lossy().to_string(),
    sylib_obj.to_string_lossy().to_string(),
    "-o".to_string(),
    ref_bin.to_string_lossy().to_string(),
    "-static".to_string(),
    "-Iscripts".to_string(),
    "-include".to_string(),
    SYLIB_HEADER.to_string(),
  ]);

  run_cmd(ref_cmd, true, None, None, Some(workspace))
    .map_err(|e| format!("reference failed: {e}"))?;

  let ref_result = run_cmd(
    vec![
      QEMU_RISCV.to_string(),
      ref_bin.to_string_lossy().to_string(),
    ],
    false,
    Some(&test_input),
    Some(Duration::from_secs(10)),
    Some(workspace),
  )
  .map_err(|e| format!("reference failed: {e}"))?;

  if kira_result.code != ref_result.code {
    return Err(format!(
      "exit code mismatch: Kira {}, Ref {}",
      kira_result.code, ref_result.code
    ));
  }

  let kira_out = kira_result.stdout.trim();
  let ref_out = ref_result.stdout.trim();

  if kira_out != ref_out {
    let kira_lines = filter_timer_lines(kira_out);
    let ref_lines = filter_timer_lines(ref_out);
    if kira_lines != ref_lines {
      return Err(format!(
        "output mismatch\nKira output:\n{}\nRef output:\n{}",
        kira_out, ref_out
      ));
    }
  }

  Ok(())
}

fn filter_timer_lines(output: &str) -> Vec<&str> {
  output
    .lines()
    .filter(|line| !line.starts_with("Timer@"))
    .collect()
}

struct CmdResult {
  stdout: String,
  code: i32,
}

fn run_cmd(
  cmd: Vec<String>,
  check: bool,
  input: Option<&str>,
  timeout: Option<Duration>,
  current_dir: Option<&Path>,
) -> Result<CmdResult, String> {
  if cmd.is_empty() {
    return Err("empty command".to_string());
  }

  let mut command = Command::new(&cmd[0]);
  command
    .args(&cmd[1..])
    .stdin(if input.is_some() {
      Stdio::piped()
    } else {
      Stdio::null()
    })
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());

  if let Some(dir) = current_dir {
    command.current_dir(dir);
  }

  let mut child = command
    .spawn()
    .map_err(|e| format!("failed to spawn `{}`: {e}", cmd.join(" ")))?;

  if let Some(stdin_data) = input
    && let Some(mut stdin) = child.stdin.take()
  {
    stdin
      .write_all(stdin_data.as_bytes())
      .map_err(|e| format!("failed to write stdin for `{}`: {e}", cmd.join(" ")))?;
  }

  let output = if let Some(timeout) = timeout {
    let start = Instant::now();
    loop {
      if child
        .try_wait()
        .map_err(|e| format!("failed to wait `{}`: {e}", cmd.join(" ")))?
        .is_some()
      {
        break child
          .wait_with_output()
          .map_err(|e| format!("failed to collect output `{}`: {e}", cmd.join(" ")))?;
      }

      if start.elapsed() > timeout {
        child
          .kill()
          .map_err(|e| format!("failed to kill timed-out command `{}`: {e}", cmd.join(" ")))?;
        let _ = child.wait();
        return Err(format!(
          "command timed out after {:?}: {}",
          timeout,
          cmd.join(" ")
        ));
      }

      thread::sleep(Duration::from_millis(20));
    }
  } else {
    child
      .wait_with_output()
      .map_err(|e| format!("failed to wait `{}`: {e}", cmd.join(" ")))?
  };

  let code = output.status.code().unwrap_or(-1);
  let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
  let stderr = String::from_utf8_lossy(&output.stderr);

  if check && !output.status.success() {
    return Err(format!(
      "command failed: {}\nstderr: {stderr}",
      cmd.join(" ")
    ));
  }

  Ok(CmdResult { stdout, code })
}

fn log(msg: &str) {
  println!("[TEST] {msg}");
}
