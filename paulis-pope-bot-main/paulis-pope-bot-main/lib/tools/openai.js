/**
 * Check if Whisper transcription is enabled
 * @returns {boolean}
 */
function isWhisperEnabled() {
  return Boolean(process.env.OPENAI_API_KEY);
}

/**
 * Transcribe audio using OpenAI Whisper API
 * @param {Buffer} audioBuffer - Audio file buffer
 * @param {string} filename - Original filename (e.g., "voice.ogg")
 * @returns {Promise<string>} Transcribed text
 */
async function transcribeAudio(audioBuffer, filename) {
  const formData = new FormData();
  formData.append('file', new Blob([audioBuffer]), filename);
  formData.append('model', 'whisper-1');

  const response = await fetch('https://api.openai.com/v1/audio/transcriptions', {
    method: 'POST',
    headers: { 'Authorization': `Bearer ${process.env.OPENAI_API_KEY}` },
    body: formData,
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`OpenAI API error: ${response.status} ${error}`);
  }

  const result = await response.json();
  return result.text;
}

export { isWhisperEnabled, transcribeAudio };
