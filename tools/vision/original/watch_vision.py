#!/usr/bin/env python3
"""
Hardened Vision Tool (v2)
- Accepts URL/File as positional arg or --url
- Retries yt-dlp downloads with fallback formats
- Handles Gemini API errors gracefully (writes failure artifact)
- Configurable via ENV variables
"""
import argparse
import os
import sys
import subprocess
import tempfile
import shutil
import time
import json
import math
from pathlib import Path

try:
    import google.generativeai as genai
except ImportError:
    genai = None

# --- Configuration ---
REPO_ROOT = Path(__file__).resolve().parents[1]
ARTIFACTS_DIR = REPO_ROOT / 'artifacts'
DEFAULT_MODEL = os.environ.get('WATCH_VISION_MODEL', 'gemini-3-flash-preview')
FALLBACK_MODEL = os.environ.get('WATCH_VISION_FALLBACK_MODEL', 'gemini-2.5-flash')
RETRIES = int(os.environ.get('WATCH_VISION_RETRIES', '2'))
CLIP_DURATION = os.environ.get('WATCH_VISION_CLIP_SECONDS', '30')
MAX_SEGMENTS = int(os.environ.get('WATCH_VISION_MAX_SEGMENTS', '20'))
UPLOAD_POLL_SECONDS = int(os.environ.get('WATCH_VISION_UPLOAD_POLL', '5'))
UPLOAD_TIMEOUT_SECONDS = int(os.environ.get('WATCH_VISION_UPLOAD_TIMEOUT', '600'))
DEFAULT_COOKIES_PATH = REPO_ROOT / "core" / "cookies.txt"

def _yt_dlp_cookie_args():
    cookie_file = os.environ.get("YTDLP_COOKIES")
    cookies_from_browser = os.environ.get("YTDLP_COOKIES_FROM_BROWSER")

    if cookie_file:
        return ["--cookies", cookie_file]
    if cookies_from_browser:
        return ["--cookies-from-browser", cookies_from_browser]
    if DEFAULT_COOKIES_PATH.exists():
        return ["--cookies", str(DEFAULT_COOKIES_PATH)]
    return []

def _yt_dlp_js_args(url):
    # Disabled: --js-runtimes not supported in apt version of yt-dlp
    # Modern yt-dlp handles YouTube without it
    return []

def _load_dotenv():
    candidates = [
        Path.cwd() / ".env",
        Path(__file__).resolve().parents[1] / ".env",
    ]
    for env_path in candidates:
        if not env_path.exists():
            continue
        try:
            for raw in env_path.read_text(encoding="utf-8").splitlines():
                line = raw.strip()
                if not line or line.startswith("#"):
                    continue
                if line.startswith("export "):
                    line = line[len("export "):].strip()
                if "=" not in line:
                    continue
                key, value = line.split("=", 1)
                key = key.strip()
                value = value.strip().strip('"').strip("'")
                if key and key not in os.environ:
                    os.environ[key] = value
        except Exception:
            continue

def _format_ts(total_seconds):
    total_seconds = max(0, int(total_seconds))
    hours = total_seconds // 3600
    minutes = (total_seconds % 3600) // 60
    seconds = total_seconds % 60
    return f"{hours:02d}:{minutes:02d}:{seconds:02d}"

def _parse_segments(raw):
    if raw is None:
        return 1
    if isinstance(raw, int):
        return max(1, raw)
    value = str(raw).strip().lower()
    if value == "all":
        return "all"
    try:
        return max(1, int(value))
    except ValueError:
        return 1

def _get_remote_duration(url):
    cmd = [
        'yt-dlp',
        '--print', '%(duration)s',
        '--no-playlist',
        *_yt_dlp_cookie_args(),
        *_yt_dlp_js_args(url),
        url
    ]
    try:
        output = subprocess.check_output(cmd, text=True).strip()
        if output:
            return float(output)
    except Exception:
        return None
    return None

def _get_local_duration(path):
    ffprobe = shutil.which("ffprobe")
    if not ffprobe:
        return None
    cmd = [
        ffprobe,
        "-v", "error",
        "-show_entries", "format=duration",
        "-of", "default=noprint_wrappers=1:nokey=1",
        path
    ]
    try:
        output = subprocess.check_output(cmd, text=True).strip()
        if output:
            return float(output)
    except Exception:
        return None
    return None

def _wait_for_active(upload):
    start = time.time()
    state = upload.state.name
    while True:
        if state == 'ACTIVE':
            return upload
        if state == 'FAILED':
            raise Exception("Gemini video processing failed")
        elapsed = time.time() - start
        if elapsed >= UPLOAD_TIMEOUT_SECONDS:
            raise Exception(f"Gemini upload not ACTIVE after {UPLOAD_TIMEOUT_SECONDS}s (state={state})")
        time.sleep(UPLOAD_POLL_SECONDS)
        upload = genai.get_file(upload.name)
        state = upload.state.name

def _clip_local_video(source_path, start_seconds, duration_seconds, tmp_dir):
    out_path = os.path.join(tmp_dir, "video.mp4")
    cmd = [
        "ffmpeg",
        "-y",
        "-ss", _format_ts(start_seconds),
        "-t", _format_ts(duration_seconds),
        "-i", source_path,
        "-c", "copy",
        out_path
    ]
    subprocess.check_call(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    return out_path

def write_artifact(filename, data):
    path = Path(filename)
    if path.is_absolute():
        path.parent.mkdir(parents=True, exist_ok=True)
    elif path.parent == Path("."):
        ARTIFACTS_DIR.mkdir(parents=True, exist_ok=True)
        path = ARTIFACTS_DIR / path
    else:
        path.parent.mkdir(parents=True, exist_ok=True)
    
    # Format as Markdown for readability
    md = f"# Vision Analysis: {data.get('title', 'Unknown')}\n\n"
    md += f"- **Source:** {data.get('source_url')}\n"
    md += f"- **Status:** {data.get('status')}\n"
    md += f"- **Model:** {data.get('model_used', 'none')}\n"
    if data.get('segments'):
        md += f"- **Segments:** {data.get('segments')}\n"
    if data.get('segment_seconds'):
        md += f"- **Segment Seconds:** {data.get('segment_seconds')}\n"
    if data.get('duration_seconds'):
        md += f"- **Duration Seconds:** {data.get('duration_seconds')}\n"
    if data.get('truncated'):
        md += f"- **Truncated:** {data.get('truncated')}\n"
    if data.get('note'):
        md += f"- **Note:** {data.get('note')}\n"
    
    if data.get('error'):
        md += f"\n## ERROR DETAIL\n```\n{data['error']}\n```\n"
        
    if data.get('analysis'):
        md += f"\n## Analysis\n{data['analysis']}\n"
        
    path.write_text(md, encoding='utf-8')
    print(f"Wrote artifact: {path}")

def download_video(url, tmp_dir, start_seconds=None, end_seconds=None):
    """Robust download with retries and format fallbacks."""
    out_tpl = os.path.join(tmp_dir, 'video.%(ext)s')

    is_hls = ".m3u8" in url
    # Try high quality first, then robust compatibility
    if is_hls:
        formats = [None, "best", "worst"]
    else:
        formats = [
            "bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]",
            "worstvideo[ext=mp4]+worstaudio[ext=m4a]/worst[ext=mp4]",
            "best",
            "worst"
        ]

    cookie_args = _yt_dlp_cookie_args()
    js_args = _yt_dlp_js_args(url)
    for fmt in formats:
        label = "default" if fmt is None else fmt
        print(f"Attempting download with format: {label}")
        cmd = [
            'yt-dlp',
            '--no-playlist',
            '--retries', str(RETRIES),
            '--fragment-retries', str(RETRIES),
            '--force-ipv4', # Stability
            *cookie_args,
            *js_args,
        ]
        if start_seconds is not None:
            end_value = end_seconds if end_seconds is not None else start_seconds + int(CLIP_DURATION)
            section = f"*{_format_ts(start_seconds)}-{_format_ts(end_value)}"
            cmd += ['--download-sections', section]
        else:
            cmd += ['--download-sections', f"*0-{CLIP_DURATION}"]  # Clip to avoid huge files
        if fmt:
            cmd += ['-f', fmt]
        cmd += ['-o', out_tpl, url]
        try:
            subprocess.check_call(cmd)
            # Find the file
            files = list(Path(tmp_dir).glob('video*'))
            if files:
                return str(files[0])
        except subprocess.CalledProcessError:
            continue
            
    raise Exception("All download formats failed.")

def _load_oauth_credentials():
    """Load OAuth credentials from Gemini CLI config."""
    oauth_path = Path.home() / ".gemini" / "oauth_creds.json"
    if not oauth_path.exists():
        return None
    try:
        import json
        from google.oauth2.credentials import Credentials
        with open(oauth_path) as f:
            data = json.load(f)
        creds = Credentials(
            token=data.get('access_token'),
            refresh_token=data.get('refresh_token'),
            token_uri="https://oauth2.googleapis.com/token",
            client_id="710733426974-0rp9b40mcj5gfnevct5fqn6aj6cmisuo.apps.googleusercontent.com",  # Gemini CLI client ID
            client_secret="",  # Public client
            scopes=data.get('scope', '').split()
        )
        return creds
    except Exception as e:
        print(f"OAuth load failed: {e}")
        return None

def analyze_with_gemini(video_path, prompt, model_name):
    if not genai:
        raise ImportError("google-generativeai package missing")

    key = os.environ.get('GEMINI_API_KEY') or os.environ.get('GOOGLE_API_KEY')
    if key:
        genai.configure(api_key=key)
    else:
        # Try OAuth credentials from Gemini CLI
        creds = _load_oauth_credentials()
        if creds:
            print("Using OAuth credentials from ~/.gemini/oauth_creds.json")
            genai.configure(credentials=creds)
        else:
            raise ValueError("GEMINI_API_KEY not found and no OAuth credentials available")
    
    print(f"Uploading {video_path} to Gemini...")
    upload = genai.upload_file(video_path)
    upload = _wait_for_active(upload)
        
    print(f"Generating content with {model_name}...")
    model = genai.GenerativeModel(model_name)
    response = model.generate_content([prompt, upload])
    return response.text

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('source', nargs='?', help="URL or File Path")
    parser.add_argument('--url', help="Alternative way to pass URL")
    parser.add_argument('--prompt', default="Analyze the Core Loop, UI, and Juice.")
    parser.add_argument('--out', default="vision_analysis.md", help="Output filename")
    parser.add_argument('--segments', help="Number of segments or 'all' to cover full duration")
    parser.add_argument('--segment-seconds', type=int, help="Override segment length in seconds")
    parser.add_argument('--segment-index', type=int, help="1-based index of a single segment to process")
    parser.add_argument('--segment-start', type=int, help="Start time in seconds for a single segment")
    args = parser.parse_args()

    _load_dotenv()

    target = args.url or args.source
    if not target:
        print("Error: No source provided.")
        sys.exit(1)

    result = {
        'source_url': target,
        'title': 'Unknown',
        'status': 'pending'
    }

    segment_seconds = int(args.segment_seconds or CLIP_DURATION)
    segments = _parse_segments(args.segments or os.environ.get('WATCH_VISION_SEGMENTS'))
    segment_start_override = None
    duration_seconds = None
    
    try:
        # 1. Determine title and duration
        if os.path.exists(target):
            result['title'] = Path(target).name
            duration_seconds = _get_local_duration(target)
        else:
            try:
                title_cmd = [
                    'yt-dlp',
                    '--print', '%(title)s',
                    '--no-playlist',
                    *_yt_dlp_cookie_args(),
                    *_yt_dlp_js_args(target),
                    target
                ]
                result['title'] = subprocess.check_output(title_cmd, text=True).strip()
            except Exception:
                pass
            duration_seconds = _get_remote_duration(target)

        # 2. Resolve segment count
        if args.segment_index:
            if args.segment_index < 1:
                result['status'] = 'download_failed'
                result['error'] = 'segment-index must be >= 1'
                write_artifact(args.out, result)
                sys.exit(0)
            segment_start_override = (args.segment_index - 1) * segment_seconds
            segments = 1
            result['note'] = f"segment-index {args.segment_index}"
        elif args.segment_start is not None:
            segment_start_override = max(0, int(args.segment_start))
            segments = 1
            result['note'] = f"segment-start {segment_start_override}s"

        if segments == "all":
            if duration_seconds:
                segments = max(1, math.ceil(duration_seconds / segment_seconds))
            else:
                segments = 1
                result['note'] = "Duration unknown; defaulted to 1 segment."
        if isinstance(segments, int) and segments > MAX_SEGMENTS:
            result['truncated'] = f"segments capped to {MAX_SEGMENTS}"
            segments = MAX_SEGMENTS

        result['segments'] = segments
        result['segment_seconds'] = segment_seconds
        if duration_seconds:
            result['duration_seconds'] = int(duration_seconds)

        # 3. Download + analyze per segment
        analyses = []
        for idx in range(int(segments)):
            if segment_start_override is not None:
                start = segment_start_override
            else:
                start = idx * segment_seconds
            if duration_seconds and start >= duration_seconds:
                result['status'] = 'download_failed'
                result['error'] = f"segment start {start}s beyond duration {int(duration_seconds)}s"
                write_artifact(args.out, result)
                sys.exit(0)
            end = start + segment_seconds
            if duration_seconds:
                end = min(end, duration_seconds)
            segment_label = f"Segment {idx + 1} ({int(start)}-{int(end)}s)"

            try:
                tmp_dir = tempfile.mkdtemp(prefix='watch_vision_')
                if os.path.exists(target):
                    video_path = _clip_local_video(target, start, end - start, tmp_dir)
                else:
                    video_path = download_video(target, tmp_dir, start_seconds=start, end_seconds=end)
            except Exception as e:
                result['status'] = 'download_failed'
                result['error'] = str(e)
                write_artifact(args.out, result)
                sys.exit(0)

            try:
                try:
                    text = analyze_with_gemini(video_path, f"{args.prompt}\n\n({segment_label})", DEFAULT_MODEL)
                    result['model_used'] = DEFAULT_MODEL
                except Exception as primary_err:
                    print(f"Primary model failed: {primary_err}. Trying fallback...")
                    text = analyze_with_gemini(video_path, f"{args.prompt}\n\n({segment_label})", FALLBACK_MODEL)
                    result['model_used'] = FALLBACK_MODEL

                analyses.append(f"### {segment_label}\n{text}")
                result['status'] = 'success'
            except Exception as e:
                result['status'] = 'analysis_failed'
                result['error'] = str(e)
                break
            finally:
                if os.path.isdir(tmp_dir):
                    shutil.rmtree(tmp_dir)

        if analyses:
            result['analysis'] = "\n\n".join(analyses)

    finally:
        pass
            
    write_artifact(args.out, result)

if __name__ == "__main__":
    main()
