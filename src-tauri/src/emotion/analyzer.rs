use std::process::Command;

pub struct EmotionAnalyzer;

#[derive(Debug)]
pub struct AnalysisResult {
    pub emotion: String,
    pub intensity: f64,
}

impl EmotionAnalyzer {
    pub fn analyze(prompt: &str) -> Option<AnalysisResult> {
        let analysis_prompt = format!(
            "Classify this prompt's emotion. Reply with ONLY a JSON object like {{\"emotion\":\"happy\",\"intensity\":0.7}}. \
             Emotions: happy (praise, gratitude), sad (frustration, anger), neutral (instructions, questions). \
             Intensity 0-1. ALL CAPS increases intensity by 0.2. Prompt: \"{}\"",
            prompt.chars().take(500).collect::<String>()
        );

        let output = Command::new("claude")
            .args(["-p", &analysis_prompt, "--model", "claude-haiku-4-5-20251001", "--output-format", "text"])
            .env_remove("CLAUDECODE")
            .output()
            .ok()?;

        let text = String::from_utf8(output.stdout).ok()?;
        let text = text.trim();

        let start = text.find('{')?;
        let end = text.rfind('}')? + 1;
        let json_str = &text[start..end];

        let parsed: serde_json::Value = serde_json::from_str(json_str).ok()?;
        let emotion = parsed["emotion"].as_str()?.to_string();
        let intensity = parsed["intensity"].as_f64().unwrap_or(0.5);

        Some(AnalysisResult { emotion, intensity })
    }
}
