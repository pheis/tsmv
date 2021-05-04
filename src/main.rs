use anyhow::{anyhow, Result};
use ropey::Rope;
use std::fs;
use std::io::BufWriter;
use std::path::PathBuf;
use structopt::StructOpt;

// use walkdir::{DirEntry, WalkDir};

mod parser;
use parser::{Lang, CST};

mod path;

mod import_string;

#[derive(StructOpt)]
struct Args {
    #[structopt(parse(from_os_str))]
    source: std::path::PathBuf,
    #[structopt(parse(from_os_str))]
    target: std::path::PathBuf,
}

fn main() -> Result<()> {
    let args = Args::from_args();

    // let text = update_imports(&args)?;

    // move_and_replace(&args, text)?;
    // find_references(&args)?;
    fs::rename(&source, &target_path)?;
    text.write_to(BufWriter::new(fs::File::create(target_path)?))?;

    Ok(())
}

fn move_and_replace(Args { target, source }: &Args, text: Rope) -> Result<()> {
    let mut target_path = target.clone();

    if target.is_dir() {
        let file_name = source.file_name().unwrap();
        target_path.push(file_name);
    }

    fs::rename(&source, &target_path)?;
    text.write_to(BufWriter::new(fs::File::create(target_path)?))?;

    Ok(())
}

fn infer_langauge_from_suffix(file_name: &PathBuf) -> Result<Lang> {
    let suffix = file_name
        .extension()
        .and_then(|os_str| os_str.to_str())
        .ok_or(anyhow!("Missing suffix on file"))?;

    match suffix {
        "ts" => Ok(Lang::TypeScript),
        "tsx" => Ok(Lang::TypeScriptTsx),
        suffix => Err(anyhow!("{:?} files are not supported", suffix)),
    }
}

fn update_imports(
    source_code: String,
    source_file: &PathBuf,
    target_file: &PathBuf,
) -> Result<String> {
    let source_dir = path::get_parent(&source_file);
    let target_dir = path::get_parent(&target_file);

    if source_dir.eq(&target_dir) {
        return Ok(source_code);
    }

    let lang = infer_langauge_from_suffix(&source_file)?;
    let mut concrete_syntax_tree = CST::new(&source_code, lang)?;

    concrete_syntax_tree.replace_all_imports(|import_string| {
        let path = import_string::to_path(&source_file, import_string)?;
        let new_import = import_string::from_paths(&target_file, &path);
        new_import
    })?;

    Ok(concrete_syntax_tree.get_source_code())
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use std::path::PathBuf;

    #[test]
    fn it_updates_imports_0() -> Result<()> {
        let code: String = r#"
            import some from '../../some';
            import other from '../../other';
            function main() {
                console.log("hullo world");
            }
            "#
        .into();

        let source: PathBuf = "/src/a/b/c/d/source.ts".into();
        let target: PathBuf = "/src/a/b/c/d/e/target.ts".into();

        let new_source_code = super::update_imports(code, &source, &target)?;

        let new_import_0: String = "import some from '../../../some';".into();
        let new_import_1: String = "import other from '../../../other';".into();

        assert!(new_source_code.contains(&new_import_0));
        assert!(new_source_code.contains(&new_import_1));
        Ok(())
    }

    #[test]
    fn it_updates_imports_1() -> Result<()> {
        let code: String = r#"
            import some from '../../some';
            import other from '../../other';
            function main() {
                console.log("hullo world");
            }
            "#
        .into();

        let source: PathBuf = "/src/a/b/c/d/source.ts".into();
        let target: PathBuf = "/src/a/target.ts".into();

        let new_source_code = super::update_imports(code, &source, &target)?;

        let new_import_0: String = "import some from './b/some';".into();
        let new_import_1: String = "import other from './b/other';".into();

        assert!(new_source_code.contains(&new_import_0));
        assert!(new_source_code.contains(&new_import_1));
        Ok(())
    }
}
