#[macro_use] extern crate lalrpop_util;

lalrpop_mod!(pub fml); // load module synthesized by LALRPOP

mod parser;
mod bytecode;

#[cfg(test)] mod tests;

use std::path::PathBuf;
use std::fs::File;
use std::io::{Read, BufReader, BufRead, Write, BufWriter};

use clap::Clap;
use clap::crate_version;
use clap::crate_authors;
use anyhow::*;

use crate::parser::AST;
use crate::fml::TopLevelParser;

use crate::bytecode::program::Program;
use crate::bytecode::serializable::Serializable;
use crate::bytecode::interpreter::evaluate_with_memory_config;

#[derive(Clap, Debug)]
#[clap(version = crate_version!(), author = crate_authors!())]
enum Action {
    Parse(ParserAction),
    Compile(CompilerAction),
    Execute(BytecodeInterpreterAction),
    Disassemble(BytecodeDisassemblyAction),
    Run(RunAction),
}

impl Action {
    pub fn execute(&self) {
        match self {
            Self::Parse(action) => action.parse(),
            Self::Compile(action) => action.compile(),
            Self::Execute(action) => action.interpret(),
            Self::Run(action) => action.run(),
            Self::Disassemble(action) => action.debug(),
        }
    }
}

#[derive(Clap, Debug)]
#[clap(about = "Run an FML program")]
struct RunAction {
    #[clap(name="FILE", parse(from_os_str))]
    pub input: Option<PathBuf>,
    #[clap(long="heap-size", name="MBs", about = "Maximum heap size in megabytes", default_value = "0")]
    pub heap_size: usize,
    #[clap(long="heap-log", name="LOG_FILE", about = "Path to heap log, if none, the log is not produced", parse(from_os_str), parse(from_os_str))]
    pub heap_log: Option<PathBuf>,
}

#[derive(Clap, Debug)]
#[clap(about = "Print FML bytecode in human-readable form")]
struct BytecodeDisassemblyAction {
    #[clap(name="FILE", parse(from_os_str))]
    pub input: Option<PathBuf>,
}

#[derive(Clap, Debug)]
#[clap(about = "Interpret FML bytecode")]
struct BytecodeInterpreterAction {
    #[clap(name="FILE", parse(from_os_str))]
    pub input: Option<PathBuf>,
    #[clap(long="heap-size", name="MBs", about = "Maximum heap size in megabytes", default_value = "0")]
    pub heap_size: usize,
    #[clap(long="heap-log", name="LOG_FILE", about = "Path to heap log, if none, the log is not produced", parse(from_os_str))]
    pub heap_log: Option<PathBuf>,
}

#[derive(Clap, Debug)]
#[clap(about = "Compiles an FML AST into bytecode")]
struct CompilerAction {
    #[clap(short = 'o', long = "output-path", alias = "output-dir", parse(from_os_str))]
    pub output: Option<PathBuf>,

    #[clap(name="FILE", parse(from_os_str))]
    pub input: Option<PathBuf>,

    #[clap(long = "output-format", alias = "bc", name = "AST_FORMAT",
    about = "The output format for the bytecode: bytes or string")]
    pub output_format: Option<BCSerializer>,

    #[clap(long = "input-format", alias = "ast", name = "BC_FORMAT",
    about = "The output format of the AST: JSON, LISP, YAML")]
    pub input_format: Option<ASTSerializer>,
}

#[derive(Clap, Debug)]
#[clap(about = "Parses FML source code and outputs an AST")]
struct ParserAction {
    #[clap(short = 'o', long = "output-path", alias = "output-dir", parse(from_os_str))]
    pub output: Option<PathBuf>,

    #[clap(name="FILE", parse(from_os_str))]
    pub input: Option<PathBuf>,

    #[clap(long = "format", alias = "ast", name = "FORMAT",
    about = "The output format of the AST: JSON, LISP, YAML, or Rust")]
    pub format: Option<ASTSerializer>,
}

macro_rules! prepare_file_path_from_input_and_serializer {
    ($self:expr) => {
        $self.output.as_ref().map(|path| {
            if path.is_dir() {
                let mut file = path.clone();
                let filename = match $self.selected_input().unwrap().name {
                    Stream::File(file) => {
                        PathBuf::from(file)
                            .file_name().unwrap()
                            .to_str().unwrap()
                            .to_owned()
                    }
                    Stream::Console => { "ast".to_owned() }
                };
                let extension = $self.selected_output_format().extension();
                file.push(filename);
                file.set_extension(extension);
                file
            } else {
                path.clone()
            }
        })
    }
}


impl RunAction {
    pub fn run(&self) {
        let source = self.selected_input()
            .expect("Cannot open FML program.");

        let ast: AST = TopLevelParser::new()
            .parse(&source.into_string()
            .expect("Error reading input"))
            .expect("Parse error");

        let program = bytecode::compile(&ast)
            .expect("Compiler error");

        evaluate_with_memory_config(&program, self.heap_size, self.heap_log.clone())
            .expect("Interpreter error")
    }

    pub fn selected_input(&self) -> Result<NamedSource> {
        NamedSource::from(self.input.as_ref())
    }
}

impl BytecodeInterpreterAction {
    pub fn interpret(&self) {
        let mut source = self.selected_input()
            .expect("Cannot open an input for the bytecode interpreter.");

        let program = BCSerializer::BYTES.deserialize(&mut source)
            .expect("Cannot parse bytecode from input.");

        evaluate_with_memory_config(&program, self.heap_size, self.heap_log.clone())
            .expect("Interpreter error")
    }

    pub fn selected_input(&self) -> Result<NamedSource> {
        NamedSource::from(self.input.as_ref())
    }
}

impl BytecodeDisassemblyAction {
    pub fn debug(&self) {
        let mut source = self.selected_input()
            .expect("Cannot open an input for the bytecode interpreter.");

        let program = BCSerializer::BYTES.deserialize(&mut source)
            .expect("Cannot parse bytecode from input.");

        println!("{}", program);
    }

    pub fn selected_input(&self) -> Result<NamedSource> {
        NamedSource::from(self.input.as_ref())
    }
}

impl CompilerAction {
    pub fn compile(&self) {
        let source = self.selected_input()
            .expect("Cannot open an input for the compiler.");
        let mut sink = self.selected_output()
            .expect("Cannot open an output for the compiler.");
        let input_serializer = self.selected_input_format()
            .expect("Cannot derive input format from file path. Consider setting it explicitly.");
        let output_serializer = self.selected_output_format();

        let source = source.into_string()
            .expect("Error reading input file");
        let ast = input_serializer.deserialize(&source)
            .expect("Error parsing AST from input file");

        let program = bytecode::compile(&ast)
            .expect("Compiler Error");

        output_serializer.serialize(&program, &mut sink)
            .expect("Cannot serialize program to output.");
    }

    pub fn selected_input(&self) -> Result<NamedSource> {
        NamedSource::from(self.input.as_ref())
    }

    pub fn selected_output_format(&self) -> BCSerializer {
        self.output_format.unwrap_or(BCSerializer::BYTES)
    }

    pub fn selected_input_format(&self) -> Option<ASTSerializer> {
        if self.input_format.is_some() {
            self.input_format
        } else {
            self.selected_input().unwrap().extension().map(|s| {
                ASTSerializer::from_extension(s.as_str())
            }).flatten()
        }
    }

    pub fn selected_output(&self) -> Result<NamedSink> {
        let maybe_file = prepare_file_path_from_input_and_serializer!(self);
        NamedSink::from(maybe_file)
    }
}

impl ParserAction {
    pub fn parse(&self) {
        let source = self.selected_input()
            .expect("Cannot open an input for the parser.");
        let mut sink = self.selected_output()
            .expect("Cannot open an output for the parser.");
        let serializer = self.selected_output_format();

        let ast: AST = TopLevelParser::new()
            .parse(&source.into_string()
            .expect("Error reading input"))
            .expect("Parse error");

        let result = serializer.serialize(&ast)
            .expect("Cannot serialize AST");

        write!(sink, "{}", result)
            .expect("Cannot write to output");
    }

    pub fn selected_input(&self) -> Result<NamedSource> {
        NamedSource::from(self.input.as_ref())
    }

    pub fn selected_output_format(&self) -> ASTSerializer {
        self.format.unwrap_or_else(|| {
            self.output.as_ref()
                .map(|path| path.extension())
                .flatten()
                .map(|extension| extension.to_str().map(|s| s.to_owned()))
                .flatten()
                .map(|extension| ASTSerializer::from_extension(extension.as_str()))
                .flatten()
                .unwrap_or(ASTSerializer::INTERNAL)
        })
    }

    pub fn selected_output(&self) -> Result<NamedSink> {
        let maybe_file = prepare_file_path_from_input_and_serializer!(self);
        NamedSink::from(maybe_file)
    }
}

#[derive(Debug, PartialOrd, PartialEq, Ord, Eq, Copy, Clone, Hash)]
enum BCSerializer {
    BYTES, STRING
}

impl BCSerializer {
    pub fn serialize(&self, program: &Program, sink: &mut NamedSink) -> Result<()> {
        match self {
            BCSerializer::BYTES  => program.serialize(sink),
            BCSerializer::STRING => unimplemented!(),
        }
    }

    pub fn deserialize(&self, source: &mut NamedSource) -> Result<Program> {
        Ok(Program::from_bytes(source))
    }

    pub fn extension(&self) -> &'static str {
        match self {
            BCSerializer::BYTES  => "bc",
            BCSerializer::STRING => "bc.txt",
        }
    }
}

impl std::str::FromStr for BCSerializer {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bytes" | "bc" | "bytecode"                  => Ok(Self::BYTES),
            "string" | "str" | "pp" | "pretty" | "print" => Ok(Self::STRING),
            format => Err(anyhow::anyhow!("Unknown BC serialization format: {}", format))
        }
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
        Ok(format!("{}\n", string))
    }

    pub fn deserialize(&self, source: &str) -> Result<AST> {
        match self {
            ASTSerializer::LISP  => Ok(serde_lexpr::from_str(source)?),
            ASTSerializer::JSON  => Ok(serde_json::from_str(source)?),
            ASTSerializer::YAML  => Ok(serde_yaml::from_str(source)?),
            ASTSerializer::INTERNAL => bail!("No deserializer implemented for Rust/INTERNAL format"),
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            ASTSerializer::LISP     => "lisp",
            ASTSerializer::JSON     => "json",
            ASTSerializer::YAML     => "yaml",
            ASTSerializer::INTERNAL => "internal",
        }
    }

    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension.to_lowercase().as_str() {
            "lisp" => Some(ASTSerializer::LISP),
            "json" => Some(ASTSerializer::JSON),
            "yaml" => Some(ASTSerializer::YAML),
            _ => None,
        }
    }
}

impl std::str::FromStr for ASTSerializer {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json"                          => Ok(Self::JSON),
            "lisp" | "sexp" | "sexpr"       => Ok(Self::LISP),
            "yaml"                          => Ok(Self::YAML),
            "rust" | "internal" | "debug"   => Ok(Self::INTERNAL),

            format => Err(anyhow::anyhow!("Unknown AST serialization format: {}", format))
        }
    }
}

#[derive(Clone,Hash,Debug,Eq,PartialEq,PartialOrd,Ord)]
enum Stream {
    File(String),
    Console,
}

impl Stream {
    #[allow(dead_code)]
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
    fn from(maybe_file: Option<&PathBuf>) -> Result<NamedSource> {
        match maybe_file {
            Some(path) => NamedSource::from_file(path),
            None => NamedSource::console(),
        }
    }
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
    fn extension(&self) -> Option<String> {
        match &self.name {
            Stream::File(file) => {
                PathBuf::from(file).extension()
                    .map(|s| s.to_str().unwrap().to_owned())
            }
            Stream::Console => None
        }
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
    fn from(maybe_file: Option<PathBuf>) -> Result<NamedSink> {
        match maybe_file {
            Some(path) => NamedSink::from_file(&path),
            None => NamedSink::console(),
        }
    }
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

    #[allow(dead_code)]
    fn extension(&self) -> Option<String> {
        match &self.name {
            Stream::File(file) => {
                PathBuf::from(file).extension()
                    .map(|s| s.to_str().unwrap().to_owned())
            }
            Stream::Console => None
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

fn main() {
    Action::parse().execute();
}