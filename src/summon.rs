use std::{collections::HashMap, io, rc::Rc};

pub trait Summoner<Obj> {
    type Id;
    type Err;
    fn summon(&self, id: Self::Id) -> Result<Obj, Self::Err>;
}

pub struct FileLocation {
    line: usize,
    column: usize,
}

pub struct FileLocationId {
    file_path: String,
    name: Rc<str>,
    location: Option<FileLocation>,
}

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
        let mut imports = HashMap::new();
        let mut decls = HashMap::new();

        for import in gosyn_file.imports {
            let name = if let Some(name) = import.name {
                name.name
            } else {
                let cells = import.path.value.split("/");
                cells.last().expect("invalid import path").to_owned()
            };

            imports.insert(name, import.path.value);
        }

        for decl in gosyn_file.decl {
            match decl {
                gosyn::ast::Declaration::Function(ref func_decl) => {
                    let name = func_decl.name.name.clone();
                    decls.insert(name, decl);
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
        Self {
            pkg_name: gosyn_file.pkg_name.name,
            line_info: gosyn_file.line_info,
            docs: gosyn_file.docs,
            imports,
            decls,
            comments: gosyn_file.comments,
        }
    }
}
