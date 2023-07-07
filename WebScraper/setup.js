const mysql = require('mysql2/promise');
require('dotenv').config();

async function SQLConnection(){
    return await mysql.createConnection({
        host: process.env.MYSQL_Host,
        user: process.env.MYSQL_User,
        password: process.env.MYSQL_Pass
    });
}
async function setup(){
    var con = await SQLConnection();
    // await con.connect();
    await con.query("create database `Ixeo`");
    await con.query("CREATE TABLE `Ixeo`.`directory` (`url` TEXT NOT NULL , `version` INT NOT NULL , `score` INT NOT NULL , `title` TEXT NOT NULL , `description` TEXT NOT NULL , `textContent` MEDIUMTEXT NOT NULL , `content` MEDIUMTEXT NOT NULL , `date` INT NOT NULL ) ENGINE = InnoDB");
    await con.query("CREATE TABLE `Ixeo`.`domains` (`domain` VARCHAR(200) NOT NULL , `score` INT NOT NULL , `date` INT NOT NULL , UNIQUE `domain` (`domain`) ) ENGINE = InnoDB");
    await con.query("CREATE TABLE `Ixeo`.`pending` (`url` TEXT NOT NULL , `date` INT NOT NULL ) ENGINE = InnoDB");
    await con.query("CREATE TABLE `Ixeo`.`queries` (`uid` INT NULL , `mid` TINYTEXT NOT NULL , `query` TINYTEXT NOT NULL , `date` INT NOT NULL ) ENGINE = InnoDB;")
    process.exit();
}

setup();