# Tauri App Icons

This directory should contain the application icons for Orkee Desktop.

## Generating Icons

To generate the required icon files, you'll need a source icon (PNG format, 1024x1024px minimum, square aspect ratio).

### Steps:

1. Create or obtain an Orkee logo in PNG format (1024x1024px recommended)
2. Run the Tauri icon generator:
   ```bash
   pnpm tauri icon path/to/your-icon.png
   ```

This will automatically generate all required formats:
- `32x32.png` - Small icon
- `128x128.png` - Medium icon  
- `128x128@2x.png` - Retina medium icon
- `icon.icns` - macOS icon bundle
- `icon.ico` - Windows icon

### Temporary Workaround

For development without icons, you can:
1. Download placeholder icons from https://icon.kitchen
2. Or create a simple colored square PNG and use the icon generator

## Icon Requirements

- **Format**: PNG with transparency
- **Size**: 1024x1024px (minimum)
- **Aspect Ratio**: 1:1 (square)
- **Color Space**: sRGB
- **Background**: Transparent recommended
