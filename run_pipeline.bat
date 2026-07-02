@echo off
setlocal

if not exist "output_imgs" mkdir "output_imgs"
if not exist "comparisons" mkdir "comparisons"

echo Running Edge Detection
for %%F in (UDED\imgs\*.png) do (
    echo Processing: %%~nxF
    .\target\release\edge-detect.exe --input "%%F" --output "output_imgs\%%~nxF"
)

echo.
echo Generating Comparisons with GT image.
.\target\release\compare-pairs.exe --left-dir output_imgs --right-dir UDED\gt --output-dir comparisons

echo.
echo Pipeline Complete! Results are in the "comparisons" folder.
pause
