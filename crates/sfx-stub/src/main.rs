use std::env;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::Command;
use tar::Archive;
use zstd::stream::read::Decoder;

const MAGIC: &[u8; 9] = b"SFX_RS_01";
const FOOTER_SIZE: usize = 8 + 9;
const CONFIG_FILENAME: &str = "__sfx_config";

#[derive(Debug, Default)]
struct Config {
    target_dir: Option<PathBuf>,
    list_only: bool,
    show_help: bool,
    verbose: bool,
}

#[derive(Debug, Clone)]
struct PayloadInfo {
    start: u64,
    size: u64,
}

fn parse_args() -> Config {
    let mut config = Config::default();
    let args: Vec<String> = env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                if i + 1 < args.len() {
                    config.target_dir = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            "-l" | "--list" => config.list_only = true,
            "-h" | "--help" => config.show_help = true,
            "-v" | "--verbose" => config.verbose = true,
            _ => {}
        }
        i += 1;
    }
    config
}

fn get_default_dir(file: &mut File, info: &PayloadInfo) -> Option<String> {
    if file.seek(SeekFrom::Start(info.start)).is_err() {
        return None;
    }
    let reader = file.take(info.size);
    let decoder = Decoder::new(reader).ok()?;
    let mut archive = Archive::new(decoder);
    
    let entry = archive.entries().ok()?.next()?;
    let mut entry = entry.ok()?;
    
    if entry.path().ok()?.as_os_str() == CONFIG_FILENAME {
        let mut content = String::new();
        if entry.read_to_string(&mut content).is_ok() {
            for line in content.lines() {
                if let Some((k, v)) = line.split_once('=') {
                    if k.trim() == "dir" {
                        return Some(v.trim().to_string());
                    }
                }
            }
        }
    }
    None
}

fn find_payload(file: &mut File) -> Result<Option<PayloadInfo>, std::io::Error> {
    let file_len = file.metadata()?.len();
    if file_len < FOOTER_SIZE as u64 {
        return Ok(None);
    }

    file.seek(SeekFrom::End(-(FOOTER_SIZE as i64)))?;
    let mut size_buf = [0u8; 8];
    file.read_exact(&mut size_buf)?;
    let archive_size = u64::from_le_bytes(size_buf);

    let mut magic_buf = [0u8; 9];
    file.read_exact(&mut magic_buf)?;

    if &magic_buf != MAGIC {
        return Ok(None);
    }

    let start = file_len - (FOOTER_SIZE as u64) - archive_size;

    Ok(Some(PayloadInfo {
        start,
        size: archive_size,
    }))
}

fn show_help(default_dir: &str) {
    println!("SFX-RS Self Extracting Archive");
    println!("Usage: <exe> [options]");
    println!("Options:");
    println!("  -o, --output <DIR>  Extract to specific directory (default: {})", default_dir);
    println!("  -l, --list          List contents only");
    println!("  -v, --verbose       Show progress");
    println!("  -h, --help          Show this help");
}

fn extract(file: &mut File, info: PayloadInfo, config: Config) -> Result<(), Box<dyn std::error::Error>> {
    file.seek(SeekFrom::Start(info.start))?;
    let reader = file.take(info.size);
    let decoder = Decoder::new(reader)?;
    let mut archive = Archive::new(decoder);

    if config.list_only {
        for entry in archive.entries()? {
            let entry = entry?;
            let path = entry.path()?;
            if path.to_str() != Some(CONFIG_FILENAME) {
                println!("{}", path.display());
            }
        }
        return Ok(());
    }

    let mut exec_cmd: Option<String> = None;
    
    let mut resolved_target_dir: Option<PathBuf> = config.target_dir.clone();
    
    let mut verbose_printed = false;
    let entries = archive.entries()?;

    for entry in entries {
        let mut entry = entry?;
        let path = entry.path()?.to_path_buf();

        if path.as_os_str() == CONFIG_FILENAME {
            let mut content = String::new();
            entry.read_to_string(&mut content)?;
            for line in content.lines() {
                if let Some((k, v)) = line.split_once('=') {
                    let k = k.trim();
                    let v = v.trim();
                    if k == "exec" {
                        exec_cmd = Some(v.to_string());
                    } else if k == "dir" {
                         if config.target_dir.is_none() {
                             resolved_target_dir = Some(PathBuf::from(v));
                         }
                    }
                }
            }
            continue;
        }
        
        let target_base = resolved_target_dir.clone().unwrap_or_else(|| PathBuf::from("."));

        if config.verbose {
             if !verbose_printed {
                println!("Extracting to: {}", target_base.display());
                verbose_printed = true;
            }
            println!("Extracting: {}", path.display());
        }

        if target_base != Path::new(".") && !target_base.exists() {
             fs::create_dir_all(&target_base)?;
        }

        entry.unpack_in(&target_base)?;
    }

    if config.verbose {
        println!("Extraction complete.");
    }

    if let Some(cmd) = exec_cmd {
        if config.verbose {
            println!("Executing: {}", cmd);
        }
        let target_base = resolved_target_dir.unwrap_or_else(|| PathBuf::from("."));
        
        #[cfg(target_os = "windows")]
        let mut child = Command::new("cmd")
            .args(["/C", &cmd])
            .current_dir(&target_base)
            .spawn()?;

        #[cfg(not(target_os = "windows"))]
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .current_dir(&target_base)
            .spawn()?;

        child.wait()?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = parse_args();
    let exe_path = env::current_exe()?;
    let mut file = File::open(&exe_path)?;

    let payload = find_payload(&mut file)?;

    if config.show_help {
        let dir = if let Some(ref info) = payload {
             get_default_dir(&mut file, info).unwrap_or_else(|| "current directory".to_string())
        } else {
             "original directory name".to_string()
        };
        show_help(&dir);
        return Ok(());
    }

    if let Some(info) = payload {
        extract(&mut file, info, config)?;
    } else if config.verbose {
        eprintln!("SFX-RS Stub: No valid archive data found.");
    }

    Ok(())
}
