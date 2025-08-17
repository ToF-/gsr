INSERT INTO Tag ( File_Path, Label ) SELECT File_Path, Label FROM Picture WHERE Length(Label) > 0 ;

