# Run the TUI program
# ./powershell_bookmarker.exe
./target/debug/powershell_bookmarker.exe

# Get the selected location, this will be '.' if it failed
$fileContent = Get-Content -Path "C:/Users/Jake/AppData/Local/Temp/powershell_script_temp_file.txt"

# CD to this location
Set-Location $fileContent
