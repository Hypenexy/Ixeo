<?php
include('../db.php');
$conn = new mysqli($host, $user, $pass);

if (mysqli_connect_errno()) exit('Connect failed: '. mysqli_connect_error());

$sql = "CREATE DATABASE `ixeoExperiment1`";
if ($conn->query($sql) === TRUE) echo 'Database successfully created<br>';

$sql = "CREATE TABLE `ixeoExperiment1`.`theinternet` ( `ip` VARCHAR(20) NOT NULL , `title` TEXT NOT NULL , `img` LONGBLOB NOT NULL , `desc` MEDIUMTEXT NOT NULL ) ENGINE = InnoDB;";
if ($conn->query($sql) === TRUE) echo 'Table successfully created<br>';

$sql = "CREATE TABLE `ixeoExperiment1`.`cachednet` ( `ip` VARCHAR(20) NOT NULL , `content` LONGTEXT NOT NULL ) ENGINE = InnoDB;";
if ($conn->query($sql) === TRUE) echo 'Table successfully created<br>';

$sql = "CREATE TABLE `ixeoExperiment1`.`status` ( `indexing` VARCHAR(20) NOT NULL, `indexingnum` VARCHAR(200) NOT NULL, `indexingnummax` VARCHAR(200) NOT NULL, `started` VARCHAR(40) NOT NULL) ENGINE = InnoDB;";
if ($conn->query($sql) === TRUE) echo 'Table successfully created<br>';

else echo 'Error: '. $conn->error . '<br>';

$conn->close();
?>