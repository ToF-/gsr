ALTER TABLE Picture DROP COLUMN Cover;
ALTER TABLE Picture ADD COLUMN Cover BOOLEAN ;
UPDATE Picture SET Cover = False ;
UPDATE Picture SET Cover = True WHERE File_Path IN (SELECT Concat(Dir_Path, '/', File_name) FROM Cover);
