.mode csv
.headers on
.output picture.csv
SELECT
File_Path AS FilePath,
Label AS Label,
File_Size AS FileSize,
Modified_Time As ModifiedTime,
Rank AS Rank,
Cover AS Cover
FROM Picture ORDER BY File_Path;
.output stdout
.quit
