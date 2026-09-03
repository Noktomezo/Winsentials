param([Parameter(Mandatory = $true)][string]$ImagePath)

$ErrorActionPreference = 'SilentlyContinue'

if (-not (Test-Path -LiteralPath $ImagePath -PathType Leaf)) {
    exit 1
}

Add-Type -AssemblyName PresentationCore
Add-Type -AssemblyName WindowsBase

$stream = $null
try {
    $stream = [System.IO.File]::OpenRead($ImagePath)
    $decoder = [System.Windows.Media.Imaging.BitmapDecoder]::Create(
        $stream,
        [System.Windows.Media.Imaging.BitmapCreateOptions]::None,
        [System.Windows.Media.Imaging.BitmapCacheOption]::OnLoad
    )
    $frame = $decoder.Frames[0]

    $data = [System.Windows.DataObject]::new()
    $data.SetImage($frame)

    # Preserve 32-bit transparency (Alpha channel) for PNG/WebP/etc.
    $stream.Position = 0
    $data.SetData("PNG", $stream, $false)

    # Allow pasting as file
    $fileDrop = [System.Collections.Specialized.StringCollection]::new()
    $fileDrop.Add((Convert-Path -LiteralPath $ImagePath)) | Out-Null
    $data.SetFileDropList($fileDrop)

    # Retry in case another process holds clipboard open
    for ($i = 0; $i -lt 10; $i++) {
        try {
            [System.Windows.Clipboard]::SetDataObject($data, $true)
            break
        }
        catch {
            Start-Sleep -Milliseconds 50
        }
    }

    [System.Media.SystemSounds]::Asterisk.Play()
}
catch {
    exit 1
}
finally {
    if ($null -ne $stream) {
        $stream.Dispose()
    }
}
