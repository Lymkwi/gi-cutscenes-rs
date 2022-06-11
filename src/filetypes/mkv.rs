pub struct MKV {
    outpath: PathBuf,
    merge_binary: PathBuf,
    command: Command
}

const GENSHIN_LANGUAGE_ORDER: [(&str, &str); 4] = [
    // Chinese is track 0
    ("chi", "Chinese (汉语)"),
    // English is track 1
    ("eng", "English"),
    // Japanese is track 2
    ("jpn", "Japanese (日本語)"),
    // Korean is track 3
    ("kor", "Korean (한국어)")
];

impl MKV {
    pub fn attempt_merge(out_path: PathBuf, v_path: PathBuf, a_paths: Vec<PathBuf>, ffmpeg_path: &str) -> GICSResult<()> {
        let mut russian_doll = MKV::new(out_path, v_path, a_paths, ffmpeg_path)?;
        let status = russian_doll.command.status()?;
        if status.success() {
            Ok(())
        } else {
            Err(GICSError::new(&format!("FFMpeg returned code {}", status.code().unwrap_or(-1))))
        }
    }

    fn new(out_path: PathBuf, v_path: PathBuf, a_paths: Vec<PathBuf>, ffmpeg_path: &str) -> GICSResult<Self> {
        // Build an argument vector
        let mut input_arguments: Vec<String> = vec!["-i".into(), v_path.to_str().unwrap().into()];
        let mut map_arguments: Vec<String> = vec!["-map".into(), "0:v".into()];
        let mut metadata_arguments: Vec<String> = Vec::new();
        let merge_arguments: Vec<String> = vec!["-c:v".into(), "copy".into(), "-c:a".into(), "libopus".into()];
        for (num, audio_path) in a_paths.iter().enumerate() {
            // Add an input argument
            input_arguments.push("-i".into());
            input_arguments.push(audio_path.to_str().unwrap().into());
            // Add the mapping of the audio
            map_arguments.push("-map".into());
            map_arguments.push(format!("{}:a", num+1));
            // Add the metadata
            let (lang_hint, lang_desc) = GENSHIN_LANGUAGE_ORDER[num];
            // First the language hint
            metadata_arguments.push(format!("-metadata:s:a:{}", num));
            metadata_arguments.push(format!("language={}", lang_hint));
            // Second the language description
            metadata_arguments.push(format!("-metadata:s:a:{}", num));
            metadata_arguments.push(format!("title=\"{}\"", lang_desc));
        }
        // Check ffmpeg_path
        let mut cmd: Command = Command::new(ffmpeg_path);
        // Modify with the arguments
        // The funny thing is that every example I found uses these methods with
        // the `Command::new` directly as builder methods.
        // But it turns out that all it does is modify the base object, and return
        // a mutable reference to potentially chain calls.
        // So, let's just modify the command we created.
        cmd.args(input_arguments)
            .args(map_arguments)
            .args(metadata_arguments)
            .args(merge_arguments)
            .arg(out_path.to_str().unwrap());

        Ok(Self {
            outpath: out_path,
            merge_binary: PathBuf::from(ffmpeg_path),
            command: cmd
        })
    }

    pub fn get_outpath(&self) -> &Path {
        self.outpath.as_path()
    }

    pub fn get_command(&self) -> &Command {
        &self.command
    }
}