<?php
$host = '192.168.1.11';
$user = 'light';
$pass = 'mide';

$conn = new mysqli($host, $user, $pass);

if (mysqli_connect_errno()) exit('Connect failed: '. mysqli_connect_error());

$sql = "CREATE DATABASE `ixeo`";
if ($conn->query($sql) === TRUE) echo 'Database successfully created<br>';

$sql = "CREATE TABLE `ixeo`.`theinternet` ( `ip` VARCHAR(20) NOT NULL , `title` TEXT NOT NULL , `img` LONGBLOB NOT NULL , `desc` MEDIUMTEXT NOT NULL ) ENGINE = InnoDB;";
if ($conn->query($sql) === TRUE) echo 'Table successfully created<br>';

$sql = "CREATE TABLE `ixeo`.`cachednet` ( `ip` VARCHAR(20) NOT NULL , `content` LONGTEXT NOT NULL ) ENGINE = InnoDB;";
if ($conn->query($sql) === TRUE) echo 'Table successfully created<br>';

else echo 'Error: '. $conn->error . '<br>';

$conn->close();
?>