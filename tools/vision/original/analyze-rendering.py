#!/usr/bin/env python3
"""
Deep analysis of vale-village-v2 overworld rendering to identify visual quality issues.
"""
from playwright.sync_api import sync_playwright
import time
import json

SERVER = "http://10.0.0.52:5174/"

def analyze_rendering():
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        context = browser.new_context(viewport={'width': 1280, 'height': 720})
        page = context.new_page()

        print("Navigating to overworld...")
        page.goto(SERVER, wait_until="networkidle", timeout=30000)
        time.sleep(2)
        page.keyboard.press("Enter")
        time.sleep(2)
        page.locator("button:has-text('New Game')").first.click()
        time.sleep(3)

        print("\n=== RENDERING PIPELINE ANALYSIS ===\n")

        # 1. Check the tilemap rendering
        print("1. TILEMAP RENDERING:")
        tilemap_info = page.evaluate("""() => {
            // Check if there's a rendering module
            const canvas = document.querySelector('.overworld-canvas');
            if (!canvas) return null;

            const ctx = canvas.getContext('2d');

            // Get image smoothing settings
            return {
                imageSmoothingEnabled: ctx.imageSmoothingEnabled,
                imageSmoothingQuality: ctx.imageSmoothingQuality,
                canvasScaling: {
                    width: canvas.width,
                    height: canvas.height,
                    styleWidth: canvas.style.width,
                    styleHeight: canvas.style.height,
                    devicePixelRatio: window.devicePixelRatio
                }
            };
        }""")
        print(json.dumps(tilemap_info, indent=3))

        # 2. Check sprite/tile source images
        print("\n2. TILE SOURCE IMAGES:")
        tile_images = page.evaluate("""() => {
            const images = performance.getEntriesByType('resource')
                .filter(r => r.name.includes('/tiles/') || r.name.includes('/sprites/'))
                .map(r => ({
                    url: r.name,
                    size: r.encodedBodySize,
                    type: r.initiatorType
                }));
            return images;
        }""")
        for img in tile_images[:20]:  # First 20
            filename = img['url'].split('/')[-1]
            print(f"   {filename}: {img['size']} bytes")

        # 3. Check canvas transform/scaling
        print("\n3. CANVAS TRANSFORM/SCALING:")
        transform_info = page.evaluate("""() => {
            const canvas = document.querySelector('.overworld-canvas');
            const container = canvas?.parentElement;

            if (!canvas) return null;

            const canvasRect = canvas.getBoundingClientRect();
            const containerRect = container?.getBoundingClientRect();

            return {
                canvas: {
                    logicalSize: `${canvas.width}x${canvas.height}`,
                    displaySize: `${Math.round(canvasRect.width)}x${Math.round(canvasRect.height)}`,
                    scale: `${(canvasRect.width / canvas.width).toFixed(3)}x`,
                    isScaled: canvasRect.width !== canvas.width || canvasRect.height !== canvas.height
                },
                container: {
                    size: `${Math.round(containerRect.width)}x${Math.round(containerRect.height)}`
                }
            };
        }""")
        print(json.dumps(transform_info, indent=3))

        # 4. Examine specific tile rendering
        print("\n4. TILE RENDERING DETAILS:")
        tile_details = page.evaluate("""() => {
            const canvas = document.querySelector('.overworld-canvas');
            if (!canvas) return null;

            const ctx = canvas.getContext('2d');

            // Sample a small area to check pixel patterns
            const sampleX = 100, sampleY = 100, sampleSize = 32;
            const imageData = ctx.getImageData(sampleX, sampleY, sampleSize, sampleSize);
            const data = imageData.data;

            // Check for interpolation artifacts (blurred edges)
            // In properly pixelated rendering, adjacent pixels should have distinct colors
            let distinctChanges = 0;
            let gradualChanges = 0;

            for (let y = 0; y < sampleSize - 1; y++) {
                for (let x = 0; x < sampleSize - 1; x++) {
                    const idx = (y * sampleSize + x) * 4;
                    const idxNext = idx + 4;

                    const rDiff = Math.abs(data[idx] - data[idxNext]);
                    const gDiff = Math.abs(data[idx+1] - data[idxNext+1]);
                    const bDiff = Math.abs(data[idx+2] - data[idxNext+2]);
                    const totalDiff = rDiff + gDiff + bDiff;

                    if (totalDiff > 50) distinctChanges++;
                    else if (totalDiff > 5 && totalDiff <= 50) gradualChanges++;
                }
            }

            return {
                sampleRegion: `${sampleX},${sampleY} to ${sampleX+sampleSize},${sampleY+sampleSize}`,
                distinctColorChanges: distinctChanges,
                gradualColorChanges: gradualChanges,
                ratio: (gradualChanges / (distinctChanges + gradualChanges)).toFixed(3),
                analysis: gradualChanges > distinctChanges * 0.3 ?
                    'LIKELY INTERPOLATED (smooth gradients detected)' :
                    'LIKELY PIXELATED (sharp edges)'
            };
        }""")
        print(json.dumps(tile_details, indent=3))

        # 5. Check rendering source code
        print("\n5. CHECKING GAME SOURCE CODE:")
        source_check = page.evaluate("""() => {
            // Try to find rendering code in window
            const checks = {};

            // Check for common game frameworks
            checks.hasPhaserRenderer = typeof window.Phaser !== 'undefined';
            checks.hasPixiRenderer = typeof window.PIXI !== 'undefined';

            // Check for custom rendering
            checks.windowKeys = Object.keys(window).filter(k =>
                k.toLowerCase().includes('render') ||
                k.toLowerCase().includes('game') ||
                k.toLowerCase().includes('canvas')
            );

            return checks;
        }""")
        print(json.dumps(source_check, indent=3))

        # 6. Network tab - check loaded images
        print("\n6. LOADED SPRITE/TILE IMAGES:")
        loaded_images = page.evaluate("""() => {
            const images = document.querySelectorAll('img');
            const canvasImages = [];

            // Also check for image elements that might be used as tile sources
            images.forEach(img => {
                if (img.src.includes('/tiles/') || img.src.includes('/overworld/')) {
                    canvasImages.push({
                        src: img.src,
                        naturalSize: `${img.naturalWidth}x${img.naturalHeight}`,
                        complete: img.complete
                    });
                }
            });

            return canvasImages;
        }""")
        for img in loaded_images[:10]:
            print(f"   {img['src'].split('/')[-1]}: {img['naturalSize']} (loaded: {img['complete']})")

        # 7. Deep dive into actual rendering code
        print("\n7. SEARCHING FOR RENDERING CODE IN SCRIPTS:")
        scripts_analysis = page.evaluate("""() => {
            const scripts = Array.from(document.querySelectorAll('script[src]'));
            return scripts.map(s => s.src).filter(src =>
                src.includes('.js') && !src.includes('node_modules')
            );
        }""")
        print(f"   Found {len(scripts_analysis)} script files")
        for script in scripts_analysis[:5]:
            print(f"   - {script.split('/')[-1]}")

        # 8. Check for any canvas scaling operations
        print("\n8. CANVAS DRAWING OPERATIONS CHECK:")
        draw_check = page.evaluate("""() => {
            const canvas = document.querySelector('.overworld-canvas');
            if (!canvas) return null;

            const ctx = canvas.getContext('2d');

            // Check current context state
            return {
                globalAlpha: ctx.globalAlpha,
                globalCompositeOperation: ctx.globalCompositeOperation,
                imageSmoothingEnabled: ctx.imageSmoothingEnabled,
                imageSmoothingQuality: ctx.imageSmoothingQuality,
                currentTransform: ctx.getTransform(),
                lineWidth: ctx.lineWidth
            };
        }""")
        print(json.dumps({
            k: str(v) if k == 'currentTransform' else v
            for k, v in draw_check.items()
        }, indent=3))

        # 9. Take multiple screenshots for comparison
        print("\n9. TAKING DETAILED SCREENSHOTS:")

        # Normal screenshot
        page.screenshot(path="/tmp/overworld-normal.png")
        print("   - /tmp/overworld-normal.png (full view)")

        # Zoomed screenshot
        page.evaluate("document.querySelector('.overworld-canvas').style.transform = 'scale(2)'")
        time.sleep(0.5)
        page.screenshot(path="/tmp/overworld-zoomed.png")
        print("   - /tmp/overworld-zoomed.png (2x zoom)")

        # Reset zoom
        page.evaluate("document.querySelector('.overworld-canvas').style.transform = 'none'")

        # Canvas only screenshot
        canvas_element = page.locator('.overworld-canvas')
        canvas_element.screenshot(path="/tmp/overworld-canvas-only.png")
        print("   - /tmp/overworld-canvas-only.png (canvas element only)")

        browser.close()
        print(f"\n✓ Analysis complete!")

if __name__ == "__main__":
    try:
        analyze_rendering()
    except Exception as e:
        print(f"\n✗ Analysis failed: {e}")
        import traceback
        traceback.print_exc()
