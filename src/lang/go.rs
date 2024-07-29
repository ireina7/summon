use std::{collections::HashMap, io, rc::Rc};

#[allow(dead_code)]
pub struct FileLocation {
    line: usize,
    column: usize,
}

#[allow(dead_code)]
pub struct FileLocationId {
    file_path: String,
    name: Rc<str>,
    location: Option<FileLocation>,
}

#[allow(dead_code)]
pub struct GoSymbolId {
    project_path: String,
    symbol: String,
}

#[allow(dead_code)]
fn summon_by_file_location(id: FileLocationId) -> io::Result<gosyn::ast::File> {
    let file = gosyn::parse_file(&id.file_path)
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

    Ok(file)
}

// #[derive(Default, Debug, Clone)]
// #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
// pub struct File {
//     pub path: PathBuf,
//     pub line_info: Vec<usize>,
//     pub docs: Vec<Rc<Comment>>,
//     pub pkg_name: Ident,
//     pub imports: Vec<Import>,
//     pub decl: Vec<Declaration>,
//     pub comments: Vec<Rc<Comment>>,
// }
pub struct GoFile {
    pub pkg_name: String,
    pub line_info: Vec<usize>,
    pub docs: Vec<Rc<gosyn::ast::Comment>>,
    pub imports: HashMap<String, String>,
    pub decls: HashMap<String, gosyn::ast::Declaration>,
    pub comments: Vec<Rc<gosyn::ast::Comment>>,
}

impl GoFile {
    pub fn from_gosyn_file(gosyn_file: gosyn::ast::File) -> Self {
        let imports = Self::extract_imports(gosyn_file.imports);
        let decls = Self::extract_decls(gosyn_file.decl);

        Self {
            pkg_name: gosyn_file.pkg_name.name,
            line_info: gosyn_file.line_info,
            docs: gosyn_file.docs,
            imports,
            decls,
            comments: gosyn_file.comments,
        }
    }

    fn extract_decls(
        decls: Vec<gosyn::ast::Declaration>,
    ) -> HashMap<String, gosyn::ast::Declaration> {
        let mut ans = HashMap::new();
        for decl in decls {
            match decl {
                gosyn::ast::Declaration::Function(ref func_decl) => {
                    let name = func_decl.name.name.clone();
                    ans.insert(name, decl);
                }
                gosyn::ast::Declaration::Type(decl) => {
                    for d in decl.specs {
                        let name = d.name.name.clone();
                        // decls.insert(name, gosyn::ast::Declaration::Type(d));
                        todo!()
                    }
                    todo!()
                }
                gosyn::ast::Declaration::Const(_) => todo!(),
                gosyn::ast::Declaration::Variable(_) => todo!(),
            };
        }
        ans
    }

    fn extract_imports(imports: Vec<gosyn::ast::Import>) -> HashMap<String, String> {
        let mut ans = HashMap::new();
        for import in imports {
            let name = if let Some(name) = import.name {
                name.name
            } else {
                let cells = import.path.value.split("/");
                cells.last().expect("invalid import path").to_owned()
            };

            ans.insert(name, import.path.value);
        }
        ans
    }
}
