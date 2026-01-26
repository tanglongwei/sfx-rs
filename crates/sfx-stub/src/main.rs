use std::env;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use tar::Archive;
use zstd::stream::read::Decoder;

const MAGIC: &[u8; 9] = b"SFX_RS_01";
const FOOTER_SIZE: usize = 8 + 9;
const CONFIG_FILENAME: &str = "__sfx_config";

struct Config {
    target_dir: PathBuf,
    list_only: bool,
    show_help: bool,
    verbose: bool,
}

fn parse_args() -> Config {
    let mut config = Config {
        target_dir: PathBuf::from("."),
        list_only: false,
        show_help: false,
        verbose: false,
    };

    let args: Vec<String> = env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                if i + 1 < args.len() {
                    config.target_dir = PathBuf::from(&args[i + 1]);
                    i += 1;
                }
            }
            "-l" | "--list" => {
                config.list_only = true;
            }
            "-h" | "--help" => {
                config.show_help = true;
            }
            "-v" | "--verbose" => {
                config.verbose = true;
            }
            _ => {}
        }
        i += 1;
    }
    config
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = parse_args();

    if config.show_help {
        println!("SFX-RS Self Extracting Archive");
        println!("Usage: <exe> [options]");
        println!("Options:");
        println!("  -o, --output <DIR>  Extract to specific directory (default: .)");
        println!("  -l, --list          List contents only");
        println!("  -v, --verbose       Show progress");
        println!("  -h, --help          Show this help");
        return Ok(());
    }

    let exe_path = env::current_exe()?;
    let mut file = File::open(&exe_path)?;

    let file_len = file.metadata()?.len();

    if file_len < FOOTER_SIZE as u64 {
        return Err("File is too small.".into());
    }

    file.seek(SeekFrom::End(-(FOOTER_SIZE as i64)))?;

    let mut size_buf = [0u8; 8];
    file.read_exact(&mut size_buf)?;
    let archive_size = u64::from_le_bytes(size_buf);

    let mut magic_buf = [0u8; 9];
    file.read_exact(&mut magic_buf)?;

    if &magic_buf != MAGIC {
        // Silent failure if it's just the stub run directly, 
        // but maybe we should print if verbose?
        if config.verbose {
            eprintln!("SFX-RS Stub: No valid archive data found.");
        }
        return Ok(());
    }

    let archive_start = file_len - (FOOTER_SIZE as u64) - archive_size;
    file.seek(SeekFrom::Start(archive_start))?;
    let limited_reader = file.take(archive_size);

    let decoder = Decoder::new(limited_reader)?;
    let mut archive = Archive::new(decoder);

    if config.list_only {
        for file in archive.entries()? {
            let file = file?;
            if let Some(name) = file.path()?.to_str() {
                if name != CONFIG_FILENAME {
                    println!("{}", name);
                }
            }
        }
        return Ok(());
    }

    if config.target_dir != Path::new(".") {
        fs::create_dir_all(&config.target_dir)?;
    }
    
    if config.verbose {
        println!("Extracting to: {}", config.target_dir.display());
    }

    // We need to iterate manually to:
    // 1. Filter out the config file (read it into memory)
    // 2. Extract others
    // 3. Show progress if verbose
    
    let mut exec_cmd: Option<String> = None;

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.to_path_buf(); // clone path
        
        // Check if config file
        if path.as_os_str() == CONFIG_FILENAME {
            let mut content = String::new();
            entry.read_to_string(&mut content)?;
            // Parse simple key=value
            for line in content.lines() {
                if let Some((k, v)) = line.split_once('=') {
                    if k.trim() == "exec" {
                        exec_cmd = Some(v.trim().to_string());
                    }
                }
            }
            continue;
        }

        if config.verbose {
            println!("Extracting: {}", path.display());
        }

        // We must ensure the path is extracted relative to target_dir
        entry.unpack_in(&config.target_dir)?;
    }

    if config.verbose {
        println!("Extraction complete.");
    }

    if let Some(cmd) = exec_cmd {
        if config.verbose {
            println!("Executing: {}", cmd);
        }
        
        // Execute command in target directory? 
        // Usually installers run in the extracted dir.
        #[cfg(target_os = "windows")]
        let mut child = Command::new("cmd")
            .args(["/C", &cmd])
            .current_dir(&config.target_dir)
            .spawn()?;

        #[cfg(not(target_os = "windows"))]
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .current_dir(&config.target_dir)
            .spawn()?;
            
        child.wait()?;
    }

    Ok(())
}