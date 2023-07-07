const mysql = require('mysql2/promise');
const puppeteer = require('puppeteer');
const sanitize = require("sanitize-filename");
const fs = require('fs');
const { type } = require('os');
require('dotenv').config();
/**
 * Use instead of console.log
 * If the first argument is 's' (for success) it will be green
 * If it's 'f' (for failure) it will be red
 */
function log(){
    if(arguments[0]=="s"){
        arguments[0] = "\x1b[32m%s\x1b[0m";
    }
    if(arguments[0]=="f"){
        arguments[0] = "\x1b[31m%s\x1b[0m";
    }
    if(arguments[0]=="server"){
        arguments[0] = "\x1b[34m\x1b[1mServer:\x1b[0m %s\x1b[0m";
        if(arguments[1]=="user"){
            arguments[1] = "\x1b[35mUser";
            arguments[2] = "\x1b[35m\x1b[1m" + arguments[2] + "\x1b[0m";
        }
    }
    console.log.apply(console, arguments);
}

async function SQLConnection(){
    return await mysql.createConnection({
        host: process.env.MYSQL_Host,
        user: process.env.MYSQL_User,
        password: process.env.MYSQL_Pass,
        database: process.env.MYSQL_DB
    });
}
async function testSQLConnection(){
    var con
    try {
        con = await SQLConnection();
        await con.connect();
    } catch (error) {
        log("f", "MySQL Database offline");
        throw error;
    }
    con.end()
    log("s", "MySQL Database online");
}

testSQLConnection()

const siteDataDir = "sitedata";
if(!fs.existsSync(siteDataDir)){
    fs.mkdirSync(siteDataDir);
    log("s", "Created folder at " + siteDataDir);
}


var con = SQLConnection();
async function ScrapeSite(URL){
    if(typeof con.query == "undefined"){
        con = await SQLConnection();
    }
    if(URL.includes("#")){
        URL = URL.split("#")[0];
    }
    const domain_Escaped = con.escape(URL.split('/')[2]);
    const checkDomainExisting = await con.query("SELECT score FROM domains WHERE domain=" + domain_Escaped);
    if(checkDomainExisting[0].length != 0){
        const newScore = checkDomainExisting[0][0].score + 10;
        const updateScore = await con.query("UPDATE domains SET score="+newScore+" WHERE domain=" + domain_Escaped);
    } else{
        const addDomain = await con.query("INSERT INTO `domains` (`domain`, `score`, `date`) VALUES ("+domain_Escaped+", 1, '"+Date.now()+"')");
    }

    const URL_Escaped = con.escape(URL);
    const checkExisting = await con.query("SELECT score FROM directory WHERE url=" + URL_Escaped);
    if(checkExisting[0].length != 0){
        const newScore = checkExisting[0][0].score + 10;
        const updateScore = await con.query("UPDATE directory SET score="+newScore+" WHERE url=" + URL_Escaped);
        return;
    }

    const browser = await puppeteer.launch({headless: 'new'});
    const page = await browser.newPage();

    await page.goto(URL);

    const title = await page.title();
    const content = await page.content();

    await page.waitForSelector('body');
    const bodyElement = await page.$('body');
    const siteText = await page.evaluate(el => el.textContent, bodyElement);

    var description
    try {
        description = await page.$eval("head > meta[name='description']", element => element.content);
    } catch (error) {
        try {
            description = await page.$eval("head > meta[name='og:description']", element => element.content);
        } catch (error) {
            description = siteText.slice(0, 40);
        }
    }

    // await page.setViewport({width: 1080, height: 1024});

    // const imageURL = siteDataDir + '/' + sanitize(URL);

    // await page.screenshot({
    //   path: imageURL,
    //   //fullPage: true,
    //   type: 'jpeg',
    //   quality: 50,
    // });

    const hrefs = await page.$$eval('a', as => as.map(a => a.href));

    for (let i = 0; i < hrefs.length; i++) {
        const link = hrefs[i];
        if(link.length>0){
            const addToPending = await con.query("INSERT INTO `pending` (`url`,`date`) VALUES ("+con.escape(link)+", '"+Date.now()+"')");
        }
    }

    await browser.close();

    const addToDirectory = await con.query("INSERT INTO `directory` (`url`, `version`, `score`, `title`, `description`, `textContent`, `content`, `date`) VALUES ("+URL_Escaped+", '1', '1', "+con.escape(title)+", "+con.escape(description)+", "+con.escape(siteText)+", "+con.escape(content)+", '"+Date.now()+"')");

    return;
}

var datasetCycles = 0; // Change to 1 to continue scraping pending links or keep as 0 to start anew with requested website.
async function CreateDataset(startingURL, iterations){
    if(typeof con.query == "undefined"){
        con = await SQLConnection();
    }
    if(datasetCycles >= iterations){
        process.exit();
    }
    datasetCycles++;

    if(datasetCycles == 1){
        await ScrapeSite(startingURL);
    }


    const getNextLink = await con.query("SELECT url FROM pending LIMIT 1");
    const deleteNextLink = await con.query("DELETE FROM pending LIMIT 1");
    if(getNextLink[0].length != 0){
        try {
            log('s', getNextLink[0][0].url)
            await ScrapeSite(getNextLink[0][0].url); // add or remove await if u don't want or want 100% CPU usage
        } catch (err) {
            log('f', getNextLink[0][0].url + ' failed because: ' + err)
        }
    }

    CreateDataset(null, iterations);
}



// ScrapeSite('https://en.wikipedia.org/');
// CreateDataset("https://nodejs.org", 20);