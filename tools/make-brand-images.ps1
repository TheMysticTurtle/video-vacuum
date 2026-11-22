<#
  Make Video Vacuum's brand images from the original art in media/.

  The originals (1-2 MB PNGs each) stay out of git (see .gitignore); this script
  makes the small copies the app, installer and README actually use, so they
  can be rebuilt any time the art changes:

    media/VideoVacuumLogoLarge-Transparent.png -> src-tauri/icons/app-icon.png  (1024x1024; then `npx tauri icon`)
    media/VideoVacuum-SkinnyBanner.png         -> src/assets/header-banner.png  (trimmed, 88 px tall: 2x of 44 px)
    media/VideoVacuumBanner-Alt.png            -> src/assets/hero-banner.png    (trimmed, 320 px tall; the "New download" hero)
    media/VideoVacuumBanner.png                -> docs/images/banner.jpg        (1280 wide, README)
                                               -> docs/images/social-preview.jpg (1280x640, GitHub social preview)
                                               -> src-tauri/installer/header.bmp (150x57, installer pages)
    media/VideoVacuumLogoLarge.png             -> src-tauri/installer/sidebar.bmp (164x314, installer welcome/finish)

  Uses only Windows' built-in .NET imaging (System.Drawing), so nothing needs
  installing. Run from anywhere:  powershell -ExecutionPolicy Bypass -File tools\make-brand-images.ps1

  -BannerGlow: VideoVacuumBanner.png stores its teal/purple glow background in
  pixels that are marked fully transparent. By default they stay transparent
  (glowing logo on dark navy); with -BannerGlow the glow background is kept.
#>
param([switch]$BannerGlow)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$media = Join-Path $root 'media'
Add-Type -AssemblyName System.Drawing

Add-Type -ReferencedAssemblies System.Drawing -TypeDefinition @'
using System;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.Drawing.Imaging;
using System.Runtime.InteropServices;

public static class BrandImages
{
    // Smallest rectangle holding every pixel more opaque than `threshold` (0-255).
    public static Rectangle OpaqueBounds(Bitmap src, int threshold)
    {
        var rect = new Rectangle(0, 0, src.Width, src.Height);
        var data = src.LockBits(rect, ImageLockMode.ReadOnly, PixelFormat.Format32bppArgb);
        var bytes = new byte[data.Stride * src.Height];
        Marshal.Copy(data.Scan0, bytes, 0, bytes.Length);
        src.UnlockBits(data);
        int minX = src.Width, minY = src.Height, maxX = -1, maxY = -1;
        for (int y = 0; y < src.Height; y++)
            for (int x = 0; x < src.Width; x++)
                if (bytes[y * data.Stride + x * 4 + 3] > threshold)
                {
                    if (x < minX) minX = x; if (x > maxX) maxX = x;
                    if (y < minY) minY = y; if (y > maxY) maxY = y;
                }
        return maxX < 0 ? rect : Rectangle.FromLTRB(minX, minY, maxX + 1, maxY + 1);
    }

    static Graphics HighQuality(Bitmap canvas)
    {
        var g = Graphics.FromImage(canvas);
        g.InterpolationMode = InterpolationMode.HighQualityBicubic;
        g.PixelOffsetMode = PixelOffsetMode.HighQuality;
        g.CompositingQuality = CompositingQuality.HighQuality;
        g.SmoothingMode = SmoothingMode.HighQuality;
        return g;
    }

    static void DrawPart(Graphics g, Bitmap src, Rectangle from, RectangleF to)
    {
        using (var attrs = new ImageAttributes())
        {
            attrs.SetWrapMode(WrapMode.TileFlipXY); // no faint border at the edges when scaling
            g.DrawImage(src, Rectangle.Round(to), from.X, from.Y, from.Width, from.Height, GraphicsUnit.Pixel, attrs);
        }
    }

    // Scale `from` (a part of src) to exactly w x h, keeping transparency.
    public static Bitmap Scale(Bitmap src, Rectangle from, int w, int h)
    {
        var canvas = new Bitmap(w, h, PixelFormat.Format32bppArgb);
        using (var g = HighQuality(canvas))
        {
            g.Clear(Color.Transparent);
            DrawPart(g, src, from, new RectangleF(0, 0, w, h));
        }
        return canvas;
    }

    // Fit src to the canvas width, centred vertically; the empty bands above and
    // below repeat the image's own top/bottom edge, so the background continues seamlessly.
    // Opaque output (for JPEG / BMP), with `matte` behind any transparency.
    public static Bitmap FitWidthExtendEdges(Bitmap src, int w, int h, Color matte)
    {
        var canvas = new Bitmap(w, h, PixelFormat.Format24bppRgb);
        using (var g = HighQuality(canvas))
        {
            g.Clear(matte);
            float scaledH = (float)src.Height * w / src.Width;
            float top = (h - scaledH) / 2f;
            if (top > 0)
            {
                DrawPart(g, src, new Rectangle(0, 0, src.Width, 1), new RectangleF(0, 0, w, top + 1));
                DrawPart(g, src, new Rectangle(0, src.Height - 1, src.Width, 1), new RectangleF(0, top + scaledH - 1, w, h - top - scaledH + 1));
            }
            DrawPart(g, src, new Rectangle(0, 0, src.Width, src.Height), new RectangleF(0, top, w, scaledH));
        }
        return canvas;
    }

    // A copy with every pixel made fully opaque (reveals colors hidden behind alpha 0).
    public static Bitmap Opaque(Bitmap src)
    {
        var copy = src.Clone(new Rectangle(0, 0, src.Width, src.Height), PixelFormat.Format32bppArgb);
        var data = copy.LockBits(new Rectangle(0, 0, copy.Width, copy.Height), ImageLockMode.ReadWrite, PixelFormat.Format32bppArgb);
        var bytes = new byte[data.Stride * copy.Height];
        Marshal.Copy(data.Scan0, bytes, 0, bytes.Length);
        for (int i = 3; i < bytes.Length; i += 4) bytes[i] = 255;
        Marshal.Copy(bytes, 0, data.Scan0, bytes.Length);
        copy.UnlockBits(data);
        return copy;
    }

    public static void SaveJpeg(Bitmap image, string path, long quality)
    {
        var codec = Array.Find(ImageCodecInfo.GetImageEncoders(), c => c.MimeType == "image/jpeg");
        using (var p = new EncoderParameters(1))
        {
            p.Param[0] = new EncoderParameter(Encoder.Quality, quality);
            image.Save(path, codec, p);
        }
    }
}
'@

function Out-Path([string]$relative) {
    $path = Join-Path $root $relative
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $path) | Out-Null
    return $path
}
function Open-Art([string]$name) {
    $path = Join-Path $media $name
    if (-not (Test-Path $path)) { throw "Missing original art: $path" }
    return [System.Drawing.Bitmap]::new($path)
}
function Report([string]$relative) {
    $file = Get-Item (Join-Path $root $relative)
    $img = [System.Drawing.Image]::FromFile($file.FullName)
    '{0,-36} {1,4}x{2,-4} {3,6:N0} KB' -f $relative, $img.Width, $img.Height, ($file.Length / 1KB)
    $img.Dispose()
}
$navy = [System.Drawing.Color]::FromArgb(255, 10, 12, 24) # matte behind any transparency in the banner

# 1. App icon source: the transparent logo at 1024x1024 (`npx tauri icon` makes every size from it).
$logo = Open-Art 'VideoVacuumLogoLarge-Transparent.png'
$icon = [BrandImages]::Scale($logo, [System.Drawing.Rectangle]::new(0, 0, $logo.Width, $logo.Height), 1024, 1024)
$icon.Save((Out-Path 'src-tauri/icons/app-icon.png'), [System.Drawing.Imaging.ImageFormat]::Png)
$icon.Dispose(); $logo.Dispose()

# 2. App header: the skinny banner trimmed to its artwork, 88 px tall (sharp at 44 px on 2x screens).
$skinny = Open-Art 'VideoVacuum-SkinnyBanner.png'
$bounds = [BrandImages]::OpaqueBounds($skinny, 24)
$height = 88
$width = [int][Math]::Round($bounds.Width * $height / $bounds.Height)
$header = [BrandImages]::Scale($skinny, $bounds, $width, $height)
$header.Save((Out-Path 'src/assets/header-banner.png'), [System.Drawing.Imaging.ImageFormat]::Png)
$header.Dispose(); $skinny.Dispose()

# 2b. "New download" hero: the Alt banner trimmed to its artwork, 320 px tall (sharp at up to 160 px on 2x screens).
$alt = Open-Art 'VideoVacuumBanner-Alt.png'
$bounds = [BrandImages]::OpaqueBounds($alt, 24)
$height = 320
$width = [int][Math]::Round($bounds.Width * $height / $bounds.Height)
$hero = [BrandImages]::Scale($alt, $bounds, $width, $height)
$hero.Save((Out-Path 'src/assets/hero-banner.png'), [System.Drawing.Imaging.ImageFormat]::Png)
$hero.Dispose(); $alt.Dispose()

# 3. README banner, GitHub social preview and the installer header strip, all from the banner.
$banner = Open-Art 'VideoVacuumBanner.png'
if ($BannerGlow) { $plain = $banner; $banner = [BrandImages]::Opaque($plain); $plain.Dispose() }
$readme = [BrandImages]::FitWidthExtendEdges($banner, 1280, [int][Math]::Round($banner.Height * 1280 / $banner.Width), $navy)
[BrandImages]::SaveJpeg($readme, (Out-Path 'docs/images/banner.jpg'), 86); $readme.Dispose()
$social = [BrandImages]::FitWidthExtendEdges($banner, 1280, 640, $navy)
[BrandImages]::SaveJpeg($social, (Out-Path 'docs/images/social-preview.jpg'), 86); $social.Dispose()
$strip = [BrandImages]::FitWidthExtendEdges($banner, 150, 57, $navy)
$strip.Save((Out-Path 'src-tauri/installer/header.bmp'), [System.Drawing.Imaging.ImageFormat]::Bmp); $strip.Dispose()
$banner.Dispose()

# 4. Installer sidebar: the logo on its dark background, centred on a tall panel.
$tile = Open-Art 'VideoVacuumLogoLarge.png'
$side = [BrandImages]::FitWidthExtendEdges($tile, 164, 314, $navy)
$side.Save((Out-Path 'src-tauri/installer/sidebar.bmp'), [System.Drawing.Imaging.ImageFormat]::Bmp); $side.Dispose()
$tile.Dispose()

'Made:'
'src-tauri/icons/app-icon.png', 'src/assets/header-banner.png', 'src/assets/hero-banner.png', 'docs/images/banner.jpg',
'docs/images/social-preview.jpg', 'src-tauri/installer/header.bmp', 'src-tauri/installer/sidebar.bmp' | ForEach-Object { Report $_ }
'Next: npx tauri icon src-tauri/icons/app-icon.png -o src-tauri/icons'
