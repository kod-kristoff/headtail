use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Result, Write};
use std::process::{Command, Stdio};
use std::time::Duration;

use tempfile::NamedTempFile;

fn number_of_input_lines() -> usize {
    let f = BufReader::new(File::open("tests/files/input.txt").unwrap());
    f.lines().count()
}

#[test]
fn help() {
    match Command::new(env!("CARGO_BIN_EXE_headtail"))
        .arg("-h")
        .output()
    {
        Ok(output) => {
            // debug_output(&output);
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("USAGE"));
            assert!(stdout.contains("headtail"));
            assert!(stdout.contains("FILENAME"));
            assert!(stdout.contains("-H"));
            assert!(stdout.contains("-T"));
        }
        Err(e) => println!("Error: {}", e),
    }
}

#[test]
fn argless() {
    match Command::new(env!("CARGO_BIN_EXE_headtail"))
        .arg("tests/files/input.txt")
        .output()
    {
        Ok(output) => {
            // debug_output(&output);
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("one\n"));
            assert!(stdout.contains("ten\n"));
            assert!(!stdout.contains("eleven\n"));
            assert!(!stdout.contains("twenty\n"));
            assert!(stdout.contains("thirty\n"));
        }
        Err(e) => println!("Error: {}", e),
    }
}

#[test]
fn head() {
    match Command::new(env!("CARGO_BIN_EXE_headtail"))
        .arg("tests/files/input.txt")
        .arg("-H")
        .arg("3")
        .arg("-T")
        .arg("0")
        .output()
    {
        Ok(output) => {
            // debug_output(&output);
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("one\n"));
            assert!(stdout.contains("two\n"));
            assert!(stdout.contains("three\n"));
            assert!(!stdout.contains("four\n"));
            assert!(!stdout.contains("twenty\n"));
            assert!(!stdout.contains("thirty\n"));
        }
        Err(e) => println!("Error: {}", e),
    }
}

#[test]
fn tail() {
    match Command::new(env!("CARGO_BIN_EXE_headtail"))
        .arg("tests/files/input.txt")
        .arg("-H")
        .arg("0")
        .arg("-T")
        .arg("3")
        .output()
    {
        Ok(output) => {
            // debug_output(&output);
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("twenty-eight\n"));
            assert!(stdout.contains("twenty-nine\n"));
            assert!(stdout.contains("thirty\n"));
            assert!(!stdout.contains("twenty-seven\n"));
            assert!(!stdout.contains("one\n"));
            assert!(!stdout.contains("ten\n"));
        }
        Err(e) => println!("Error: {}", e),
    }
}

#[test]
fn head_length_exceeds_file_length() {
    let expected_lines = number_of_input_lines();
    let head_lines = 64;
    assert!(expected_lines < head_lines, "Expected input file to have fewer than {} lines (otherwise this test doesn't test anything)", head_lines);

    match Command::new(env!("CARGO_BIN_EXE_headtail"))
        .arg("tests/files/input.txt")
        .arg("-H")
        .arg(head_lines.to_string())
        .arg("-T")
        .arg("0")
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let num_lines = stdout.lines().count();
            assert_eq!(num_lines, expected_lines);
        }
        Err(e) => println!("Error: {}", e),
    }
}

#[test]
fn tail_length_exceeds_file_length() {
    let expected_lines = number_of_input_lines();
    let tail_lines = 64;
    assert!(expected_lines < tail_lines, "Expected input file to have fewer than {} lines (otherwise this test doesn't test anything)", tail_lines);

    match Command::new(env!("CARGO_BIN_EXE_headtail"))
        .arg("tests/files/input.txt")
        .arg("-H")
        .arg("0")
        .arg("-T")
        .arg(tail_lines.to_string())
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let num_lines = stdout.lines().count();
            assert_eq!(num_lines, expected_lines);
        }
        Err(e) => println!("Error: {}", e),
    }
}

#[test]
fn overlapping_head_and_tail() {
    let expected_lines = number_of_input_lines();
    let head_lines = 20;
    let tail_lines = 20;
    assert!(expected_lines < head_lines + tail_lines, "Expected input file to have fewer than {} lines (otherwise this test doesn't test anything)", head_lines + tail_lines);

    match Command::new(env!("CARGO_BIN_EXE_headtail"))
        .arg("tests/files/input.txt")
        .arg("-H")
        .arg(head_lines.to_string())
        .arg("-T")
        .arg(tail_lines.to_string())
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let num_lines = stdout.lines().count();
            assert_eq!(num_lines, expected_lines);
        }
        Err(e) => println!("Error: {}", e),
    }
}

// TODO: Add test for -f/--follow

#[test]
fn follow_detects_recreation() -> Result<()> {
    // create a temporary file
    // Write
    println!("1");
    let mut tmpfile = NamedTempFile::new()?;
    write!(tmpfile, "first file\n")?;
    let tmpfilename = tmpfile.into_temp_path();
    println!("2");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_headtail"))
        .arg(&tmpfilename)
        .arg("--follow")
        .stdout(Stdio::piped())
        .spawn()?;
    println!("3");

    std::fs::remove_file(&tmpfilename)?;
    println!("4");

    let mut newfile = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(tmpfilename)?;
    println!("5");

    write!(newfile, "second file\n")?;
    println!("6");

    let _ = newfile.flush();
    drop(newfile);
    println!("7");

    std::thread::sleep(Duration::from_millis(100));
    cmd.kill()?;
    println!("8");

    match cmd.wait_with_output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("AAAAAAAAA {} BBBBBBBB {}", stdout, output.status.success());
            assert!(stdout.contains("first file"));
            assert!(stdout.contains("second file"));
        }
        Err(e) => println!("Error: {}", e),
    }
    Ok(())
}
