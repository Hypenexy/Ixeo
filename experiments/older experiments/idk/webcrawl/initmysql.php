<?php
include('../../db.php');
$conn = new mysqli($host, $user, $pass);

$sql = "DROP DATABASE `ixeo`";
if ($conn->query($sql) === TRUE) echo 'Database successfully deleted<br>';

$sql = "CREATE DATABASE `ixeo`";
if ($conn->query($sql) === TRUE) echo 'Database successfully created<br>';

$sql = "CREATE TABLE `ixeo`.`cachednet` ( `url` VARCHAR(600) NOT NULL , `content` LONGTEXT NOT NULL ) ENGINE = InnoDB;";
if ($conn->query($sql) === TRUE) echo 'Table successfully created<br>';

$sql = "CREATE TABLE `ixeo`.`links` ( `url` VARCHAR(600) NOT NULL , `links` LONGTEXT NOT NULL ) ENGINE = InnoDB;";
if ($conn->query($sql) === TRUE) echo 'Table successfully created<br>';

$sql = "CREATE TABLE `ixeo`.`status` ( `indexing` VARCHAR(600) NOT NULL , `started` VARCHAR(40) NOT NULL) ENGINE = InnoDB;";
if ($conn->query($sql) === TRUE) echo 'Table successfully created<br>';
