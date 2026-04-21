param(
    [string[]]$Path = @(".\ng_selfhost_clean.exe", ".\output.exe")
)

$ErrorActionPreference = "Stop"

function Read-U16 {
    param([byte[]]$Bytes, [int]$Offset)
    return [BitConverter]::ToUInt16($Bytes, $Offset)
}

function Read-U32 {
    param([byte[]]$Bytes, [int]$Offset)
    return [BitConverter]::ToUInt32($Bytes, $Offset)
}

function Get-PeLayout {
    param([string]$FilePath)

    $fullPath = (Resolve-Path -LiteralPath $FilePath).Path
    $bytes = [System.IO.File]::ReadAllBytes($fullPath)
    if ($bytes.Length -lt 0x100) {
        throw "PE short: $FilePath"
    }

    $peOffset = [int](Read-U32 $bytes 0x3C)
    if ($peOffset -lt 0 -or $peOffset + 0xF8 -gt $bytes.Length) {
        throw "PE header bad: $FilePath"
    }
    if ((Read-U32 $bytes $peOffset) -ne 0x4550) {
        throw "PE sig bad: $FilePath"
    }

    $sectionCount = [int](Read-U16 $bytes ($peOffset + 6))
    $optHeaderSize = [int](Read-U16 $bytes ($peOffset + 20))
    $entryRva = [int](Read-U32 $bytes ($peOffset + 0x28))
    $sizeOfImage = [int](Read-U32 $bytes ($peOffset + 0x50))
    $importRva = [int](Read-U32 $bytes ($peOffset + 0x90))
    $sectionOffset = $peOffset + 24 + $optHeaderSize

    $sections = @{}
    for ($i = 0; $i -lt $sectionCount; $i++) {
        $base = $sectionOffset + 40 * $i
        if ($base + 40 -gt $bytes.Length) {
            throw "PE sections bad: $FilePath"
        }
        $name = [System.Text.Encoding]::ASCII.GetString($bytes[$base..($base + 7)]).Trim([char]0)
        $sections[$name] = [pscustomobject]@{
            Name = $name
            VirtualSize = [int](Read-U32 $bytes ($base + 8))
            VirtualAddress = [int](Read-U32 $bytes ($base + 12))
            RawSize = [int](Read-U32 $bytes ($base + 16))
            RawPointer = [int](Read-U32 $bytes ($base + 20))
        }
    }

    foreach ($name in @(".text", ".rdata", ".idata", ".bss")) {
        if (-not $sections.ContainsKey($name)) {
            throw "PE section missing ${name}: $FilePath"
        }
    }

    [pscustomobject]@{
        Path = $fullPath
        Size = $bytes.Length
        EntryRva = $entryRva
        SizeOfImage = $sizeOfImage
        ImportRva = $importRva
        Text = $sections[".text"]
        Rdata = $sections[".rdata"]
        Idata = $sections[".idata"]
        Bss = $sections[".bss"]
    }
}

$expectedTextRva = 0x1000
$expectedRdataRva = 0x101000
$expectedReservedRdataSize = 0x4000
$expectedIdataRva = $expectedRdataRva + $expectedReservedRdataSize
$expectedBssRva = $expectedIdataRva + 0x1000
$expectedImportRva = $expectedIdataRva
$expectedIdataRawSize = 0x400
$expectedIdataVirtualSize = 0x200
$expectedBssSize = 0x10000
$checked = 0

foreach ($item in $Path) {
    $layout = Get-PeLayout -FilePath $item

    if ($layout.Text.VirtualAddress -ne $expectedTextRva) {
        throw "PE text rva bad: $item"
    }
    if ($layout.Rdata.VirtualAddress -ne $expectedRdataRva) {
        throw "PE rdata rva bad: $item"
    }
    if ($layout.Idata.VirtualAddress -ne $expectedIdataRva) {
        throw "PE idata rva bad: $item"
    }
    if ($layout.Bss.VirtualAddress -ne $expectedBssRva) {
        throw "PE bss rva bad: $item"
    }
    if ($layout.ImportRva -ne $expectedImportRva) {
        throw "PE import rva bad: $item"
    }
    if ($layout.Idata.RawSize -ne $expectedIdataRawSize) {
        throw "PE idata raw bad: $item"
    }
    if ($layout.Idata.VirtualSize -ne $expectedIdataVirtualSize) {
        throw "PE idata vsize bad: $item"
    }
    if ($layout.Bss.VirtualSize -ne $expectedBssSize) {
        throw "PE bss size bad: $item"
    }
    if ($layout.Rdata.VirtualSize -ne $expectedReservedRdataSize) {
        throw "PE rdata vsize bad: $item"
    }
    if ($layout.Rdata.RawSize -gt $expectedReservedRdataSize) {
        throw "PE rdata overflow: $item"
    }

    $rdataHeadroom = $expectedReservedRdataSize - $layout.Rdata.RawSize
    Write-Output ("PE LAYOUT {0} size={1} entry=0x{2:X} rdata_vsize=0x{3:X} rdata_raw=0x{4:X} headroom_floor={5} idata=0x{6:X} bss=0x{7:X}" -f `
        $item, `
        $layout.Size, `
        $layout.EntryRva, `
        $layout.Rdata.VirtualSize, `
        $layout.Rdata.RawSize, `
        $rdataHeadroom, `
        $layout.Idata.VirtualAddress, `
        $layout.Bss.VirtualAddress)
    $checked++
}

Write-Output ("PE LAYOUT PASS {0}" -f $checked)
