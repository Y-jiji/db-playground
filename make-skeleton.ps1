foreach ($x in "db-core", "db-side", "db-test", "db-typing") 
{ cd $x; cargo clean; cd .. }
xcopy /s . ..\db-x