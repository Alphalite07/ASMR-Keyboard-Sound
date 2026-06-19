import librosa
import soundfile as sf
import numpy as np
import os

# 1. Load your clustered audio file
audio_file = "cluster.wav" # <-- CHANGE THIS to your file name
print(f"Loading {audio_file}...")

# sr=None preserves the original quality and sample rate of your audio
y, sr = librosa.load(audio_file, sr=None)

print("Hunting for energy spikes (transients)...")

# 2. Detect the exact frames where a "spike" starts
# backtrack=True tells the algorithm to find the exact beginning of the sound, 
# rather than the absolute peak of the volume.
onset_frames = librosa.onset.onset_detect(y=y, sr=sr, backtrack=True)

# Convert frames to exact audio samples (array indices)
onset_samples = librosa.frames_to_samples(onset_frames)

# Add the very end of the file to the list so we can slice the final click
onset_samples = np.append(onset_samples, len(y))

if not os.path.exists("sounds"):
    os.makedirs("sounds")

print(f"Found {len(onset_samples) - 1} individual clicks! Extracting...")

# 3. Slice the audio from one spike to the next
for i in range(len(onset_samples) - 1):
    start = onset_samples[i]
    
    # We want to slice until the *next* spike starts. 
    # However, to keep files small and snappy for your Rust app, 
    # we'll cap the max length of a click to 250 milliseconds.
    max_length = int(sr * 0.25) 
    end = min(onset_samples[i+1], start + max_length)
    
    click_audio = y[start:end]
    
    # Optional but highly recommended: Apply a tiny fade-out to the end of the slice.
    # This prevents an audio "pop" or "click" artifact when the audio suddenly cuts off.
    fade_length = int(sr * 0.01) # 10ms fade out
    if len(click_audio) > fade_length:
        fade_curve = np.linspace(1.0, 0.0, fade_length)
        click_audio[-fade_length:] *= fade_curve
        
    # Export the isolated spike
    out_file = f"sounds/click_{i}.wav"
    sf.write(out_file, click_audio, sr)

print("Extraction complete. Check the 'sounds' folder.")