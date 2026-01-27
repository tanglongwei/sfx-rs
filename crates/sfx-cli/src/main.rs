use anyhow::{Context, Result};
use clap::Parser;
use console::style;
use std::fs::{File};
use std::io::{Write};
use std::path::{PathBuf};
use tar::{Builder, Header};
use walkdir::WalkDir;
use zstd::stream::write::Encoder;


const STUB_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/sfx-stub.bin"));
const MAGIC: &[u8; 9] = b"SFX_RS_01";
const CONFIG_FILENAME: &str = "__sfx_config";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: PathBuf,

    #[arg(short, long)]
    output: Option<PathBuf>,

    #[arg(short, long, default_value_t = 3)]
    level: i32,

    #[arg(short = 'e', long)]
    exec: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    let output_path = match &args.output {
        Some(p) => p.clone(),
        None => {
            let input_path = args.input.canonicalize().unwrap_or_else(|_| args.input.clone());
            
            let stem = input_path.file_name()
                .filter(|s| !s.is_empty())
                .or_else(|| {
                    if input_path.as_os_str().len() > 1 {
                        Some(std::ffi::OsStr::new("output"))
                    } else {
                        Some(std::ffi::OsStr::new("output"))
                    }
                })
                .unwrap_or(std::ffi::OsStr::new("output"));

            let mut p = PathBuf::from(stem);
            #[cfg(target_os = "windows")]
            p.set_extension("exe");
            
            #[cfg(not(target_os = "windows"))]
            if p.extension().is_none() {
                p.set_extension("sfx");
            }
            p
        }
    };

    println!("{} Creating SFX archive...", style("[1/3]").bold().green());
    println!("  Input: {}", args.input.display());
    println!("  Output: {}", output_path.display());
    
    let mut out_file = File::create(&output_path).context("Failed to create output file")?;
    out_file.write_all(STUB_BYTES).context("Failed to write stub to output file")?;
    out_file.flush()?; 
    
    let stub_len = out_file.metadata()?.len();
    
    println!("{} Compressing data...", style("[2/3]").bold().green());

    {
        let encoder = Encoder::new(&mut out_file, args.level)?;
        
        let mut tar = Builder::new(encoder);
        
        let mut config_content = String::new();
        if let Some(cmd) = &args.exec {
            config_content.push_str(&format!("exec={}\n", cmd));
        }

        let input_path = args.input.canonicalize().unwrap_or(args.input.clone());
        
        if input_path.is_file() {
            if !config_content.is_empty() {
                let mut header = Header::new_gnu();
                header.set_size(config_content.len() as u64);
                header.set_mode(0o644);
                header.set_cksum();
                tar.append_data(&mut header, CONFIG_FILENAME, config_content.as_bytes())?;
            }

            let file_name = input_path.file_name().unwrap();
            let mut f = File::open(&input_path)?;
            tar.append_file(file_name, &mut f).context("Failed to append file")?;
        } else if input_path.is_dir() {
            if let Some(dir_name) = input_path.file_name().and_then(|n| n.to_str()) {
                config_content.push_str(&format!("dir={}\n", dir_name));
            }

            if !config_content.is_empty() {
                let mut header = Header::new_gnu();
                header.set_size(config_content.len() as u64);
                header.set_mode(0o644);
                header.set_cksum();
                tar.append_data(&mut header, CONFIG_FILENAME, config_content.as_bytes())?;
            }

            let walker = WalkDir::new(&input_path);
            for entry in walker {
                let entry = entry?;
                let path = entry.path();
                
                let relative_path = path.strip_prefix(&input_path)?;
                
                if relative_path.as_os_str().is_empty() {
                    continue; 
                }

                if path.is_dir() {
                    tar.append_dir(relative_path, path).context("Failed to append dir")?;
                } else {
                    let mut f = File::open(path)?;
                    tar.append_file(relative_path, &mut f).context("Failed to append file")?;
                }
            }
        } else {
            return Err(anyhow::anyhow!("Input path does not exist"));
        }
        
        tar.finish().context("Failed to finish tar")?;
        let encoder = tar.into_inner()?;
        encoder.finish().context("Failed to finish zstd")?;
    } 

    println!("{} Finalizing SFX...", style("[3/3]").bold().green());
    
    let final_len = out_file.metadata()?.len();
    let archive_size = final_len - stub_len;

    out_file.write_all(&archive_size.to_le_bytes())?;
    out_file.write_all(MAGIC)?;

    println!("{} Done! created {}", style("Success!").bold().green(), output_path.display());
    println!("  Stub size: {} bytes", stub_len);
    println!("  Archive size: {} bytes", archive_size);
    println!("  Total size: {} bytes", final_len + 17);

    Ok(())
}