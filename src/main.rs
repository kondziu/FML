#[macro_use] extern crate lalrpop_util;

lalrpop_mod!(pub fml); // load module synthesized by LALRPOP

mod parser;

use std::path::PathBuf;
use std::collections::HashMap;
use std::borrow::Borrow;
use std::fs::{create_dir_all, File};
use std::io::{Read, BufReader, BufRead, Write, BufWriter};

use clap::Clap;
use anyhow::*;

use crate::parser::AST;
use crate::fml::TopLevelParser;

#[derive(Clap, Debug)]
#[clap(about = "Parses FML source code and outputs an AST")]
struct ParserAction {
    #[clap(short = 'o', long = "output-path", alias = "output-dir", parse(from_os_str))]
    pub output_path: Option<PathBuf>,

    #[clap(name="FILE", parse(from_os_str))]
    pub inputs: Vec<PathBuf>,

    #[clap(long = "as-json")]
    pub json: bool,

    #[clap(long = "as-yaml")]
    pub yaml: bool,

    #[clap(long = "as-sexpr", alias = "as-lisp")]
    pub lisp: bool,

    #[clap(long = "as-internal", alias = "as-internal")]
    pub internal: bool,
}

impl ParserAction {
    pub fn parse(&self) {
        let mut outputs: HashMap<(Stream, ASTSerializer), NamedSink> = self.selected_outputs()
            .expect("Error creating outputs for parser")
            .into_iter()
            .map(|(key, output)| (key, output.expect("Error creating output")))
            .collect();

        for input in self.selected_inputs() {
            let source: NamedSource = input
                .expect("Error creating input");

            let source_name = source.name.clone();

            let ast: AST = TopLevelParser::new()
                .parse(&source.into_string().expect("Error reading input"))
                .expect("Parse error");

            // FIXME the extensions need to be set for different serializers too D:
            let serializers: Vec<ASTSerializer> = self.selected_ast_serializers();
            for serializer in serializers {
                let sink_key = (source_name.clone(), serializer);
                let sink = outputs.get_mut(&sink_key)
                    .expect(&format!("No output found for input source {:?}", &source_name));

                let ast_string = serializer.serialize(&ast)
                    .expect("Could not serialize ASt to string");
                write!(sink, "{}", ast_string)
                    .expect(&format!("Error writing AST to \"{:?}\"", sink.name));
            }
        }
    }

    pub fn selected_ast_serializers(&self) -> Vec<ASTSerializer> {
        let mut serializers = Vec::new();
        if self.json     { serializers.push(ASTSerializer::JSON)     }
        if self.yaml     { serializers.push(ASTSerializer::YAML)     }
        if self.lisp     { serializers.push(ASTSerializer::LISP)     }
        if self.internal { serializers.push(ASTSerializer::INTERNAL) }
        if serializers.is_empty() { serializers.push(ASTSerializer::INTERNAL) }
        serializers
    }

    pub fn selected_inputs(&self) -> Vec<Result<NamedSource>> {
        if self.inputs.is_empty() {
            vec![NamedSource::console()]
        } else {
            self.inputs.iter().map(|path: &PathBuf| NamedSource::from_file(path)).collect()
        }
    }

    pub fn selected_outputs(&self) -> Result<HashMap<(Stream, ASTSerializer), Result<NamedSink>>> {
        #[allow(dead_code)]
        fn filename_from_path(path: &PathBuf) -> Result<String> {
            if path.is_dir() {
                bail!("Expected file, but {:?} is a directory.", path);
            }
            path.file_name().map(|os_str| os_str.to_str()).flatten()
                .map_or(Err(anyhow!("Cannot extract filename from path {:?}.", path)),
                        |string| Ok(string.to_owned()))
        }

        enum Inputs<'a> { Empty, One(&'a PathBuf), Many(&'a [PathBuf]) }
        let inputs = match self.inputs.len() {
            0 => Inputs::Empty,
            1 => Inputs::One(self.inputs.last().unwrap()),
            _ => Inputs::Many(self.inputs.borrow()),
        };

        let serializers = self.selected_ast_serializers();
        // let extensions: Vec<String> = self.selected_ast_serializers()
        //     .into_iter()
        //     .map(|serializer| serializer.extension().to_owned())
        //     .collect();

        let mut map: HashMap<(Stream, ASTSerializer), Result<NamedSink>> = HashMap::new();
        match (inputs, self.output_path.as_ref()) {
            (Inputs::Empty, None) => {
                for serializer in serializers {
                    map.insert((Stream::Console, serializer), NamedSink::console());
                }
            },

            (Inputs::One(input_path), None) => {
                for serializer in serializers{
                    let key = (Stream::from(input_path)?, serializer);
                    map.insert(key, NamedSink::console());
                }
            },

            (Inputs::Many(paths), None) => {
                for serializer in serializers {
                    for input_path in paths {
                        let key = (Stream::from(input_path)?, serializer);
                        map.insert(key, NamedSink::console());
                    }
                }
            },

            (Inputs::Empty, Some(output_path)) => {
                let mut output_path = output_path.clone();
                if output_path.is_dir() {
                    output_path.push("ast");
                }
                for serializer in serializers {
                    let key = (Stream::Console, serializer);
                    let mut output_path = output_path.clone();
                    output_path.set_extension(serializer.extension().to_owned());
                    map.insert(key, NamedSink::from_file(&output_path));
                }
            }

            (Inputs::One(path), Some(output_path)) => {
                let mut output_path = output_path.clone();
                if output_path.is_dir() {
                    if let Some(filename) = path.file_name() {
                        output_path.push(filename);
                    } else {
                        bail!("Cannot extract file name from path {:?}", path);
                    }
                }
                for serializer in serializers {
                    let key = (Stream::from(path)?, serializer);
                    let mut output_path = output_path.clone();
                    output_path.set_extension(serializer.extension().to_owned());
                    map.insert(key, NamedSink::from_file(&output_path));
                }
            }

            (Inputs::Many(paths), Some(dir)) if dir.is_dir() || !dir.exists() => {
                create_dir_all(dir)?;
                for path in paths {
                    let mut output_path = dir.clone();
                    if let Some(filename) = path.file_name() {
                        output_path.push(filename);
                    } else {
                        bail!("Cannot extract file name from path {:?}", path);
                    }
                    for serializer in serializers.iter() {
                        let key = (Stream::from(path)?, serializer.clone());
                        let mut output_path = output_path.clone();
                        output_path.set_extension(serializer.extension().to_owned());
                        map.insert(key, NamedSink::from_file(&output_path));
                    }
                }
            }

            (Inputs::Many(paths), Some(output_path)) => {
                bail!("Expected output path {:?} to be a directory since there are several inputs: {:?}.",
                      output_path, paths)
            }
        };

        return Ok(map);
    }
}

#[derive(Debug, PartialOrd, PartialEq, Ord, Eq, Copy, Clone, Hash)]
enum ASTSerializer {
    LISP, JSON, YAML, INTERNAL
}
impl ASTSerializer {
    pub fn serialize(&self, ast: &AST) -> Result<String> {
        let string = match self {
            ASTSerializer::LISP  => serde_lexpr::to_string(&ast)?,
            ASTSerializer::JSON  => serde_json::to_string(&ast)?,
            ASTSerializer::YAML  => serde_yaml::to_string(&ast)?,
            ASTSerializer::INTERNAL => format!("{:?}", ast),
        };
        Ok(format!("{}\n",string))
    }
    pub fn extension(&self) -> &str {
        match self {
            ASTSerializer::LISP     => "lisp",
            ASTSerializer::JSON     => "json",
            ASTSerializer::YAML     => "yaml",
            ASTSerializer::INTERNAL => "internal",
        }
    }
}

#[derive(Clone,Hash,Debug,Eq,PartialEq,PartialOrd,Ord)]
enum Stream {
    File(String),
    Console,
}
impl Stream {
    pub fn from(path: &PathBuf) -> Result<Self> {
        if let Some(str) = path.as_os_str().to_str() {
            Ok(Stream::File(str.to_owned()))
        } else {
            Err(anyhow!("Cannot convert path into UTF string: {:?}", path))
        }
    }
}

struct NamedSource {
    name: Stream,
    source: Box<dyn BufRead>
}
impl NamedSource {
    fn console() -> Result<NamedSource> {
        let named_source = NamedSource {
            name: Stream::Console,
            source: Box::new(BufReader::new(std::io::stdin())),
        };
        Ok(named_source)
    }
    fn from_file(path: &PathBuf) -> Result<Self> {
        if let Some(name) = path.as_os_str().to_str() {
            File::open(path).map(|file| NamedSource {
                name: Stream::File(name.to_owned()),
                source: Box::new(BufReader::new(file)),
            }).map_err(|error| anyhow!("Cannot open file for reading \"{}\": {}", name, error))
            // TODO maybe directories too?
        } else {
            bail!("Cannot convert path into UTF string: {:?}", path)
        }
    }
    fn into_string(mut self) -> Result<String> {
        let mut string = String::new();
        self.source.read_to_string(&mut string)?;
        Ok(string)
    }
}
impl Read for NamedSource {
    fn read(&mut self, buf: &mut [u8]) -> std::result::Result<usize, std::io::Error> {
        self.source.read(buf)
    }
}
impl BufRead for NamedSource {
    fn fill_buf(&mut self) -> std::result::Result<&[u8], std::io::Error> {
        self.source.fill_buf()
    }
    fn consume(&mut self, amt: usize) {
        self.source.consume(amt)
    }
}
impl std::fmt::Debug for NamedSource {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("<")?;
        match &self.name {
            Stream::File(file) => f.write_str(&file),
            Stream::Console => f.write_str("stdin"),
        }?;
        Ok(())
    }
}

struct NamedSink {
    name: Stream,
    sink: Box<dyn Write>,
}
impl NamedSink {
    fn console() -> Result<Self> {
        let named_sink = NamedSink {
            name: Stream::Console,
            sink: Box::new(std::io::stdout()),
        };
        Ok(named_sink)
    }
    fn from_file(path: &PathBuf) -> Result<Self> {
        if let Some(name) = path.as_os_str().to_str() {
            File::create(path).map(|file| NamedSink {
                name: Stream::File(name.to_owned()),
                sink: Box::new(BufWriter::new(file)),
            }).map_err(|error| anyhow!("Cannot open file for writing \"{}\": {}", name, error))
        } else {
            bail!("Cannot convert path into UTF string: {:?}", path)
        }
    }
}
impl Write for NamedSink {
    fn write(&mut self, buf: &[u8]) -> std::result::Result<usize, std::io::Error> {
        self.sink.write(buf)
    }
    fn flush(&mut self) -> std::result::Result<(), std::io::Error>{
        self.sink.flush()
    }
}

impl std::fmt::Debug for NamedSink {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(">")?;
        match &self.name {
            Stream::File(file) => f.write_str(&file),
            Stream::Console => f.write_str("stout"),
        }?;
        Ok(())
    }
}

#[derive(Clap, Debug)]
#[clap(version = "1.0", author = "Konrad Siek <konrad.siek@gmail.com>")]
enum Action {
    Parse(ParserAction)
}

impl Action {
    pub fn execute(&self) {
        match self {
            Self::Parse(action) => action.parse()
        }
    }
}

fn main() {
    Action::parse().execute();
}