$ErrorActionPreference = "Stop"

$archive = "cp-cli-$env:TARGET.zip"
New-Item -ItemType Directory -Force -Path dist/package, dist/smoke | Out-Null
Copy-Item "target/$env:TARGET/release/$env:EXECUTABLE" "dist/package/$env:EXECUTABLE"
Copy-Item README.md, LICENSE dist/package/
Compress-Archive -Path dist/package/* -DestinationPath "dist/$archive"
Expand-Archive -Path "dist/$archive" -DestinationPath dist/smoke -Force
$actualVersion = & dist/smoke/cp-cli.exe --version
if ($actualVersion -ne "cp-cli $env:VERSION") {
  throw "Expected cp-cli $env:VERSION, received $actualVersion"
}
"path=dist/$archive" | Out-File -FilePath $env:GITHUB_OUTPUT -Append -Encoding utf8
