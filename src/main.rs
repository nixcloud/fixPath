use std::{
    ffi::{CStr, CString},
    fs::{self, OpenOptions},
    io::{Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    process,
};

use anyhow::{anyhow, bail};
use clap::{Parser, Subcommand};
use colored::Colorize;
use object::{
    pe,
    read::{
        coff::CoffHeader,
        pe::{fixpath, ImageNtHeaders},
        SectionIndex,
    },
    FileKind, LittleEndian,
};

struct RequestChangeSet {
    from: String,
    to: String,
}

#[derive(Debug)]
struct MakeChangeSet {
    dll_changes: Vec<ChangeImport>,
}

#[derive(Debug)]
struct Import {
    dll_name: String,
    abs_address: u32,
}

#[derive(Debug)]
struct ChangeImport {
    original_dll_name: String, // fixPath entry
    old_dll_name: String,      // the old override, same as original_dll_name usually
    new_dll_name: String,      // the next override
    abs_address: u32,          // where to make the override
}

#[derive(Debug)]
struct FixPathSectionInfo {
    version: u32,
    fix_path_size: u32,
    idata_entries: Vec<String>,
    didata_entries: Vec<String>,
}

#[derive(Debug)]
struct FixPathData {
    imports: Vec<Import>,
    delayed_imports: Vec<Import>,
    info: FixPathSectionInfo,
}

/// Modify FS locations of linked DLLs in an PE executable
#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Lists DLL/delayed DLL imports
    ListImports {
        /// Path to the executable
        file: PathBuf,
    },

    /// Updates DLL bindings
    SetImport {
        /// The executable to be modified
        exe: PathBuf,
        /// The DLL name
        dll_name: PathBuf,
        /// The absolute path to the DLL
        target_path: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    match Args::parse().command {
        Command::ListImports { file } => process_imports(&file, None),
        Command::SetImport {
            exe,
            dll_name,
            target_path,
        } => {
            let from = dll_name.display().to_string();
            let to = target_path.display().to_string();
            let dll_change = RequestChangeSet { from, to };
            process_imports(&exe, Some(dll_change))
        }
    }
}

fn process_imports(
    in_file_path: &Path,
    dll_change: Option<RequestChangeSet>,
) -> anyhow::Result<()> {
    println!(
        "TARGET: \n - {}",
        in_file_path.display().to_string().yellow()
    );

    let in_file = match fs::File::open(&in_file_path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Failed to open file '{}': {err}", in_file_path.display());
            process::exit(1);
        }
    };
    let in_data = match unsafe { memmap2::Mmap::map(&in_file) } {
        Ok(mmap) => mmap,
        Err(err) => {
            eprintln!("Failed to map file '{}': {err}", in_file_path.display());
            process::exit(1);
        }
    };
    let in_data = &*in_data;

    let kind = match FileKind::parse(in_data) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Failed to parse file: {}", err);
            process::exit(1);
        }
    };

    let make_change_set = match kind {
        FileKind::Pe32 => process_file::<pe::ImageNtHeaders32>(in_data, dll_change),
        FileKind::Pe64 => process_file::<pe::ImageNtHeaders64>(in_data, dll_change),
        _ => {
            eprintln!("Not a PE file");
            process::exit(1);
        }
    };

    let mut file = OpenOptions::new().write(true).open(&in_file_path)?;

    let Some(make_change_set) = make_change_set? else {
        return Ok(());
    };

    for cs in make_change_set.dll_changes {
        if cs.old_dll_name == cs.new_dll_name {
            continue;
        }
        file.seek(SeekFrom::Start(cs.abs_address as u64))?;
        // Convert Rust String into CString
        let c_string = CString::new(cs.new_dll_name.clone())?;

        // Convert CString into &CStr
        let c_str: &CStr = c_string.as_c_str();
        // FIXME maybe we should reset all fiels to 0 which are not covered by a string
        file.write_all(c_str.to_bytes_with_nul())
            .map_err(|err| anyhow!("unable to write make_change_set to file: {err}"))?;
        if cs.original_dll_name == cs.new_dll_name {
            println!("UPDATE {} @ 0x{:0x}", cs.new_dll_name, cs.abs_address);
        } else {
            println!(
                "UPDATE {} @ 0x{:0x} -> {} (modified)",
                cs.old_dll_name.red().strikethrough(),
                cs.abs_address,
                cs.new_dll_name.green()
            );
        }
    }
    println!("DONE");
    Ok(())
}

fn process_file<Pe: ImageNtHeaders>(
    in_data: &[u8],
    dll_change: Option<RequestChangeSet>,
) -> anyhow::Result<Option<MakeChangeSet>> {
    let fix_path_data: FixPathData;
    let fix_path_section_info: FixPathSectionInfo;

    let in_dos_header = pe::ImageDosHeader::parse(in_data)?;
    let mut nt_headers_offset = in_dos_header.nt_headers_offset().into();
    let (in_nt_headers, in_data_directories) = Pe::parse(in_data, &mut nt_headers_offset)?;
    let in_file_header = in_nt_headers.file_header();
    let in_sections = in_file_header.sections(in_data, nt_headers_offset)?;

    let Some(import_table) = in_data_directories.import_table(in_data, &in_sections)? else {
        bail!("no import table found");
    };
    let mut import_descriptor_iterator = import_table.descriptors()?;

    let fix_path_section: Option<(SectionIndex, &pe::ImageSectionHeader)> =
        in_sections.enumerate().find(|(_, section)| {
            let s = String::from_utf8_lossy(&section.name);
            s == ".fixPath"
        });

    match fix_path_section {
        Some(p) => {
            let offset = p.1.pointer_to_raw_data.get(LittleEndian);
            let fix_path_section = fixpath::parse(in_data, offset)?;
            let version = fix_path_section.header.version.get(LittleEndian);
            let fix_path_size = fix_path_section.header.fix_path_size.get(LittleEndian);
            let idata_name_table_size = fix_path_section
                .header
                .idata_name_table_size
                .get(LittleEndian);
            let didata_name_table_size = fix_path_section
                .header
                .didata_name_table_size
                .get(LittleEndian);

            let mut offset = (offset + 16) as usize;
            let idata_entries = fixpath::read_fixpath_import_dll_names(
                in_data,
                &mut offset,
                idata_name_table_size,
            )?;
            let didata_entries = fixpath::read_fixpath_import_dll_names(
                in_data,
                &mut offset,
                didata_name_table_size,
            )?;

            fix_path_section_info = FixPathSectionInfo {
                version,
                fix_path_size,
                idata_entries,
                didata_entries,
            };
        }
        None => {
            eprintln!(" - {}", "No .fixPath section found in PE executable!".red());
            process::exit(1);
        }
    }
    // read imports
    let mut imports: Vec<Import> = vec![];
    // FIXME import_descriptor_iterator could be empty
    while let Ok(Some(import)) = import_descriptor_iterator.next() {
        let dll_name_address: u32 = import.name.get(LittleEndian); // e74
        let dll_name_abs_address =
            import_table.name_address(dll_name_address) + import_table.section_offset();
        let dll_name = std::str::from_utf8(import_table.name(dll_name_address)?)?;
        imports.push(Import {
            dll_name: String::from(dll_name),
            abs_address: dll_name_abs_address,
        });
    }

    // read delayed imports
    let mut delayed_imports: Vec<Import> = vec![];
    let Some(delayed_import_table) =
        in_data_directories.delay_load_import_table(in_data, &in_sections)?
    else {
        bail!("no delay load import table found");
    };
    let mut delayed_import_descriptor_iterator = delayed_import_table.descriptors()?;
    // FIXME handle unwrap on files without delay imports
    while let Ok(Some(delayed_import)) = delayed_import_descriptor_iterator.next() {
        let dll_name_address: u32 = delayed_import.dll_name_rva.get(LittleEndian);
        let dll_name_abs_address =
            import_table.name_address(dll_name_address) + import_table.section_offset();
        let dll_name = std::str::from_utf8(delayed_import_table.name(dll_name_address)?)?;

        delayed_imports.push(Import {
            dll_name: String::from(dll_name),
            abs_address: dll_name_abs_address,
        });
    }

    fix_path_data = FixPathData {
        imports,
        delayed_imports,
        info: fix_path_section_info,
    };

    if fix_path_data.imports.len() != fix_path_data.info.idata_entries.len() {
        eprintln!("The .fixPath's import section claims it knows ({}) imports but the PE header has ({}), something is wrong!",
                  fix_path_data.imports.len(), fix_path_data.info.idata_entries.len());
        process::exit(1);
    }

    if fix_path_data.delayed_imports.len() != fix_path_data.info.didata_entries.len() {
        eprintln!("The .fixPath's delayed imports section claims it knows ({}) imports but the PE header has ({}), something is wrong!",
                  fix_path_data.delayed_imports.len(), fix_path_data.info.didata_entries.len());
        process::exit(1);
    }

    let Some(change) = dll_change else {
        println!(" - fixPath version: {}", fix_path_data.info.version);
        println!(" - fix_path_size: {}", fix_path_data.info.fix_path_size);
        println!();
        println!("IMPORTS");
        fn print_dll_unmodified(index: usize, fix_path_dll_name: String, abs_address: u32) {
            println!(
                " - {}, {} @ 0x{:0x}",
                index + 1,
                fix_path_dll_name,
                abs_address
            );
        }
        fn print_dll_modified(
            index: usize,
            fix_path_dll_name: String,
            abs_address: u32,
            imports_dll_name: String,
        ) {
            println!(
                " - {}, {} @ 0x{:0x} -> {} (modified)",
                index + 1,
                fix_path_dll_name.red().strikethrough(),
                abs_address,
                imports_dll_name.green()
            );
        }
        for (i, num) in fix_path_data.imports.iter().enumerate() {
            let fix_path_dll_name = fix_path_data.info.idata_entries[i].clone();
            let imports_dll_name = num.dll_name.clone();
            if fix_path_dll_name == imports_dll_name {
                print_dll_unmodified(i, fix_path_dll_name, num.abs_address);
            } else {
                print_dll_modified(i, fix_path_dll_name, num.abs_address, imports_dll_name);
            }
        }
        println!("DELAYED IMPORTS");
        for (i, num) in fix_path_data.delayed_imports.iter().enumerate() {
            let fix_path_dll_name = fix_path_data.info.didata_entries[i].clone();
            let imports_dll_name = num.dll_name.clone();
            if fix_path_dll_name == imports_dll_name {
                print_dll_unmodified(i, fix_path_dll_name, num.abs_address);
            } else {
                print_dll_modified(i, fix_path_dll_name, num.abs_address, imports_dll_name);
            }
        }
        println!();
        return Ok(None);
    };

    println!();

    if change.to.len() + 1 >= fix_path_data.info.fix_path_size as usize {
        eprintln!("Can't update DLL name because new name is {} and max size, including terminator 0, is {}!",
                  change.to.len()+1, fix_path_data.info.fix_path_size);
        process::exit(1);
    }
    let mut make_change_set: MakeChangeSet = MakeChangeSet {
        dll_changes: vec![],
    };
    fn try_find_in_vec(v: &Vec<String>, from: &String) -> Option<usize> {
        v.iter().position(|el| el == from)
    }
    match try_find_in_vec(&fix_path_data.info.idata_entries, &change.from) {
        Some(i) => {
            let old_dll_name = fix_path_data.imports[i].dll_name.clone();
            let original_dll_name = fix_path_data.imports[i].dll_name.clone();
            let new_dll_name = change.to.clone();
            let abs_address = fix_path_data.imports[i].abs_address;
            // println!("CHANGE IMPORTS\n - {} @ 0x{:0x} -> {}", old_dll_name.red().strikethrough(),
            //           abs_address, new_dll_name.green());
            make_change_set.dll_changes.push(ChangeImport {
                abs_address,
                original_dll_name,
                old_dll_name,
                new_dll_name,
            })
        }
        None => {}
    }
    match try_find_in_vec(&fix_path_data.info.didata_entries, &change.from) {
        Some(i) => {
            let old_dll_name = fix_path_data.delayed_imports[i].dll_name.clone();
            let original_dll_name = fix_path_data.delayed_imports[i].dll_name.clone();
            let new_dll_name = change.to.clone();
            let abs_address = fix_path_data.delayed_imports[i].abs_address;
            // println!("CHANGE DELAYED IMPORTS\n - {} @ 0x{:0x} -> {}", old_dll_name.red().strikethrough(),
            //           abs_address, new_dll_name.green());
            make_change_set.dll_changes.push(ChangeImport {
                abs_address,
                original_dll_name,
                old_dll_name,
                new_dll_name,
            })
        }
        None => {}
    }
    if make_change_set.dll_changes.len() > 0 {
        return Ok(Some(make_change_set));
    }
    eprintln!(
        "Can't find the DLL '{}' in the IMPORTS/DELAYED IMPORTS of PE file",
        change.from
    );
    process::exit(1);
}
