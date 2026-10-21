param([Parameter(Mandatory)][string]$Path, [Parameter(Mandatory)][string]$Fixture)
# Independent schema-1 reader. .NET BinaryReader integer primitives are little-endian.
$ErrorActionPreference = 'Stop'
$raw = [IO.File]::ReadAllBytes((Resolve-Path -LiteralPath $Path))
if ($raw.Length -gt 16777216) { throw 'Checkpoint exceeds limit' }
$stream = [IO.MemoryStream]::new($raw,$false)
$reader = [IO.BinaryReader]::new($stream,[Text.UTF8Encoding]::new($false,$true))
function Read-Text([int]$Limit) {
    $length = $reader.ReadUInt32()
    if ($length -gt $Limit -or $length -gt ($stream.Length-$stream.Position)) { throw 'Invalid string length' }
    $bytes = $reader.ReadBytes([int]$length)
    return [Text.UTF8Encoding]::new($false,$true).GetString($bytes)
}
function Read-Frame {
    $text = Read-Text 65536
    $anchor = $reader.ReadUInt32(); $caret = $reader.ReadUInt32()
    if ($anchor -gt [Text.Encoding]::UTF8.GetByteCount($text) -or $caret -gt [Text.Encoding]::UTF8.GetByteCount($text)) { throw 'Invalid anchor' }
    return @{text=$text;selection=@($anchor,$caret)}
}
function Read-History {
    $count = $reader.ReadUInt32()
    if ($count -gt 32) { throw 'Invalid history count' }
    $frames = @(); for ($i=0; $i -lt $count; $i++) { $frames += Read-Frame }
    return ,$frames
}
function Read-Bool {
    $value = $reader.ReadByte()
    if ($value -gt 1) { throw 'Invalid boolean' }
    return [bool]$value
}
try {
    if ([Text.Encoding]::ASCII.GetString($reader.ReadBytes(8)) -cne 'COSDOC01' -or $reader.ReadUInt16() -ne 1) { throw 'Unsupported checkpoint' }
    if (($reader.ReadBytes(3) -join '.') -ne '17.0.0') { throw 'Unsupported Unicode data' }
    if ($reader.ReadUInt32() -ne $raw.Length) { throw 'Length mismatch' }
    $modelRevision=$reader.ReadUInt64(); $id=$reader.ReadUInt64(); $documentRevision=$reader.ReadUInt64()
    $frame=Read-Frame; $saved=Read-Text 65536; $undo=Read-History; $redo=Read-History
    $scroll=$reader.ReadUInt32(); $query=Read-Text 4096
    $themeByte=$reader.ReadByte(); if ($themeByte -gt 2) { throw 'Invalid theme' }
    $theme=@('Light','Dark','HighContrast')[$themeByte]; $size=$reader.ReadUInt16()
    $motion=Read-Bool; $effects=Read-Bool
    $window=@(); for ($i=0; $i -lt 4; $i++) { $window += $reader.ReadInt32() }
    $focus=Read-Text 64; $fileScroll=$reader.ReadUInt32(); $selectedFile=$reader.ReadUInt32()
    if ($stream.Position -ne $stream.Length) { throw 'Trailing bytes' }
    $state=@{text=$frame.text;document_revision=$documentRevision;model_revision=$modelRevision;selection=$frame.selection;dirty=($frame.text -cne $saved);query=$query;scroll=$scroll;theme=$theme;text_size=$size;reduced_motion=$motion;reduced_effects=$effects;window=$window;focus=$focus;file_scroll=$fileScroll;selected_file=$selectedFile;undo=$undo.Count;redo=$redo.Count}
    $expected=(Get-Content -LiteralPath $Fixture -Raw | ConvertFrom-Json -AsHashtable).final
    foreach ($key in $expected.Keys) {
        if ($key -eq 'matches') { continue } # Search is an executor assertion, not a stored field.
        if ((ConvertTo-Json -InputObject $state[$key] -Compress) -cne (ConvertTo-Json -InputObject $expected[$key] -Compress)) { throw "Independent checkpoint mismatch: $key" }
    }
    if ($id -ne 1 -or $saved -cne '' -or $undo.Count -ne 2 -or $redo.Count -ne 1 -or $undo[0].text -cne "Draft: unsaved`n" -or $undo[1].text -cne "Draft: notes`n" -or $redo[0].text -cne "Draft: `n") { throw 'Independent history mismatch' }
    @{status='passed';reader='PowerShell/.NET independent field decoder';bytes=$raw.Length;sha256=(Get-FileHash -LiteralPath $Path).Hash.ToLowerInvariant();state=$state;undo_frames=$undo;redo_frames=$redo}
} finally { $reader.Dispose(); $stream.Dispose() }
