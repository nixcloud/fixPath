mod cargo_env;
use cargo_env::{VERSION, get_executable_name};

use clap::{Arg, ArgAction, Command};

use std::{fs, process};
use std::io::Error;
use colored::Colorize;
use object::{LittleEndian, pe};
use object::read::coff::CoffHeader;
use object::read::pe::{fixpath, ImageNtHeaders};
use object::read::{SectionIndex};

struct DllFix {
    filename: String,
    from: String,
    to: String,
}

#[derive(Debug)]
struct Import {
    dll_name: String,
    abs_address: u32,
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

fn main() {
    let matches = Command::new("{NAME}")
        .about(format!(">>> {} to modify FS locations of linked DLLs in an PE executable <<<", get_executable_name()))
        .arg(
            Arg::new("version")
                .long("version")
                .help("Prints the version")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("list-imports")
                .long("list-imports")
                .short('l')
                .help(format!("Lists DLL/delayed DLL imports loads of <file>, {} -l test.exe", get_executable_name()))
                .value_name("arg")
        )
        .arg(
            Arg::new("set-import")
                .long("set-import")
                .short('s')
                .help(format!("Updates DLL <file> bindings for <from> so it points to <to>, {} -s test.exe foo.dll c:\\foo.dll", get_executable_name()))
                .value_name("arg")
                .num_args(3)
                .required(false),
        )
        .group(
            clap::ArgGroup::new("commands")
                .args(&["version", "list-imports", "set-import"])
                .required(true)
                .multiple(false),
        )
        .get_matches();

    if matches.get_flag("version") {
        println!("{} version {}", get_executable_name(), VERSION);
    } else if let Some(filename) = matches.get_one::<String>("list-imports") {
        process_imports(filename, None);
    } else if let Some(values) = matches.get_many::<String>("set-import") {
        let args: Vec<&str> = values.map(|s| s.as_str()).collect();
        // println!("set-import: {}, {}, {}", args[0], args[1], args[2]);
        // let dll_change = DLLChange { from: args[1], to: args[2]};

        let dll_change = DllFix {
            filename: String::from(args[0]),
            from: String::from(args[1]),
            to: String::from(args[2])
        };
        process_imports(args[0], Some(dll_change));
    }
}

fn process_imports(in_file_path: &str, dll_change: Option<DllFix>) {
    println!("TARGET: \n - {}", in_file_path.yellow());

    let in_file = match fs::File::open(&in_file_path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Failed to open file '{}': {}", in_file_path, err,);
            process::exit(1);
        }
    };
    let in_data = match unsafe { memmap2::Mmap::map(&in_file) } {
        Ok(mmap) => mmap,
        Err(err) => {
            eprintln!("Failed to map file '{}': {}", in_file_path, err,);
            process::exit(1);
        }
    };
    let in_data = &*in_data;

    let kind = match object::FileKind::parse(in_data) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Failed to parse file: {}", err);
            process::exit(1);
        }
    };
    let _out_data = match kind {
        object::FileKind::Pe32 => process_file::<pe::ImageNtHeaders32>(in_data, dll_change),
        object::FileKind::Pe64 => process_file::<pe::ImageNtHeaders64>(in_data, dll_change),
        _ => {
            eprintln!("Not a PE file");
            process::exit(1);
        }
    };
}

fn process_file<Pe: ImageNtHeaders>(in_data: &[u8], dll_change: Option<DllFix>)
    -> Result<(), object::Error> {

    let fix_path_data: FixPathData;
    let fix_path_section_info: FixPathSectionInfo;

    let in_dos_header = pe::ImageDosHeader::parse(in_data)?;
    let mut nt_headers_offset = in_dos_header.nt_headers_offset().into();
    // let in_rich_header = object::read::pe::RichHeaderInfo::parse(in_data, offset);
    let (in_nt_headers, in_data_directories) = Pe::parse(in_data, &mut nt_headers_offset)?;
    let in_file_header = in_nt_headers.file_header();
    // let in_optional_header = in_nt_headers.optional_header();
    let in_sections = in_file_header.sections(in_data, nt_headers_offset)?;

    let import_table = in_data_directories.import_table(in_data, &in_sections)?.unwrap();
    let mut import_descriptor_iterator = import_table.descriptors()?;

    let fix_path_section: Option<(SectionIndex, &pe::ImageSectionHeader)> = in_sections.enumerate()
        .find(|(_, section)| {
            let s = String::from_utf8_lossy(&section.name);
            s == ".fixPath"
        });

    match fix_path_section {
        Some(p) => {
            let offset = p.1.pointer_to_raw_data.get(LittleEndian);
            let fix_path_section = fixpath::parse(in_data, offset)?;
            let version = fix_path_section.header.version.get(LittleEndian);
            let fix_path_size = fix_path_section.header.fix_path_size.get(LittleEndian);
            let idata_name_table_size = fix_path_section.header.idata_name_table_size.get(LittleEndian);
            let didata_name_table_size = fix_path_section.header.didata_name_table_size.get(LittleEndian);

            let mut offset = (offset + 16) as usize;
            let idata_entries = fixpath::read_import_strings(in_data, &mut offset, idata_name_table_size)?;
            let didata_entries = fixpath::read_import_strings(in_data, &mut offset, didata_name_table_size)?;

            fix_path_section_info = FixPathSectionInfo {
                version,
                fix_path_size,
                idata_entries,
                didata_entries,
            };
        },
        None => {
            eprintln!(" - {}", "No .fixPath section found in PE executable!".red());
            process::exit(1);
        },
    }

    // read **dllName records**
    let mut imports: Vec<Import> = vec![];
    // FIXME import_descriptor_iterator could be empty
    while let Some(import) = import_descriptor_iterator.next().unwrap() {
        let dll_name_address: u32 = import.name.get(LittleEndian); // e74
        let dll_name_abs_address =import_table.name_address(dll_name_address) + import_table.section_offset();
        let dll_name = std::str::from_utf8(import_table.name(dll_name_address)?).unwrap();
        imports.push(Import { dll_name: String::from(dll_name), abs_address: dll_name_abs_address });
    }

    // read delayed dllName records
    let mut delayed_imports: Vec<Import> = vec![];
    let delayed_import_table = in_data_directories.delay_load_import_table(in_data, &in_sections)?.unwrap();
    // FIXME handle unwrap on files without delay imports
    let mut delayed_import_descriptor_iterator = delayed_import_table.descriptors()?;
    while let Some(delayed_import) = delayed_import_descriptor_iterator.next().unwrap() {
        //println!("{:?}", import);
        let dll_name_address: u32 = delayed_import.dll_name_rva.get(LittleEndian);
        let dll_name_abs_address = import_table.name_address(dll_name_address) + import_table.section_offset();
        let dll_name = std::str::from_utf8(delayed_import_table.name(dll_name_address)?).unwrap();

        delayed_imports.push(Import { dll_name: String::from(dll_name), abs_address: dll_name_abs_address });
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
            println!(" - {}, {} @ 0x{:0x}", index+1,  fix_path_dll_name, abs_address);
        }
        fn print_dll_modified(index: usize, fix_path_dll_name: String, abs_address: u32, imports_dll_name: String) {
            println!(" - {}, {} @ 0x{:0x} -> {} (modified)", index+1,
                     fix_path_dll_name.red().strikethrough(), abs_address, imports_dll_name.green());
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
        return Ok(())
    };

    println!();

    if change.to.len() + 1 >= fix_path_data.info.fix_path_size as usize {
        eprintln!("Can't update DLL name because new name is {} and max size, including terminator 0, is {}!",
                  change.to.len()+1, fix_path_data.info.fix_path_size);
        process::exit(1);
    }

    fn try_find_in_vec(v: &Vec<Import>, from: &String) -> Option<usize> {
        v.iter().position(|el| el.dll_name == *from)
    }

    match try_find_in_vec(&fix_path_data.imports, &change.from) {
        Some(i) => {
            println!("CHANGE IMPORTS\n - {} @ 0x{:0x} -> {}", change.from.red().strikethrough(),
                     fix_path_data.imports[i].abs_address, change.to.green());
            return Ok(())
        },
        None => {}
    }

    match try_find_in_vec(&fix_path_data.delayed_imports, &change.from) {
        Some(i) => {
            println!("CHANGE DELAYED IMPORTS\n - {} @ 0x{:0x} -> {}", change.from.red().strikethrough(),
                     fix_path_data.imports[i].abs_address, change.to.green());
            return Ok(())
        },
        None => {}
    }

    eprintln!("Can't find the DLL '{}' in the IMPORTS/DELAYED IMPORTS of PE file", change.from);
    process::exit(1);
}
