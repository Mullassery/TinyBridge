#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SOURCE_PNG="$SCRIPT_DIR/AppIcon-source.png"
ICONSET_DIR="$SCRIPT_DIR/AppIcon.iconset"
OUTPUT_ICNS="$SCRIPT_DIR/AppIcon.icns"

# If no source PNG exists, generate a placeholder programmatically
if [ ! -f "$SOURCE_PNG" ]; then
    echo "Generating placeholder app icon..."

    # Use sips to create a simple 1024x1024 PNG
    # This creates a basic dark square with a light border (developer tools blue theme)
    python3 << 'EOF'
import os
from PIL import Image, ImageDraw

output_path = os.environ.get('SOURCE_PNG', 'AppIcon-source.png')
size = 1024
bg_color = (59, 89, 152)  # developer-tools blue
accent_color = (200, 220, 255)  # light accent

img = Image.new('RGB', (size, size), bg_color)
draw = ImageDraw.Draw(img)

# Draw a light border/frame
border = 60
draw.rectangle(
    [(border, border), (size - border, size - border)],
    outline=accent_color,
    width=8
)

# Add a simple box in the center
inner_margin = 200
draw.rectangle(
    [(inner_margin, inner_margin), (size - inner_margin, size - inner_margin)],
    fill=accent_color,
    outline=accent_color
)

img.save(output_path, 'PNG')
print(f"Created placeholder icon: {output_path}")
EOF
fi

echo "Building .iconset from $SOURCE_PNG..."

# Create iconset directory
mkdir -p "$ICONSET_DIR"

# Standard macOS icon sizes (in pixels)
declare -a sizes=(16 32 64 128 256 512 1024)

for size in "${sizes[@]}"; do
    output="$ICONSET_DIR/icon_${size}x${size}.png"
    sips -z "$size" "$size" "$SOURCE_PNG" --out "$output" 2>/dev/null || true

    # Create @2x variant for sizes that need it
    if [ "$size" -lt 512 ]; then
        double=$((size * 2))
        output_2x="$ICONSET_DIR/icon_${size}x${size}@2x.png"
        sips -z "$double" "$double" "$SOURCE_PNG" --out "$output_2x" 2>/dev/null || true
    fi
done

echo "Converting .iconset to .icns..."
iconutil -c icns "$ICONSET_DIR" -o "$OUTPUT_ICNS"

# Clean up the iconset directory
rm -rf "$ICONSET_DIR"

echo "✓ Created $OUTPUT_ICNS"
