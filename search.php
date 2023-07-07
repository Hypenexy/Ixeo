<?php
if(isset($_GET["q"])){
    $q = $_GET["q"];
    if($q==""){
        header("Location: ./");
    }
}
else{
    header("Location: ./");
}
?>
<!DOCTYPE html>
<html>
<head><meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0, user-scalable=0">
    <title><?php echo $q?> - Ixeo</title>
    <link rel="stylesheet" href="search.css">
    <?php if(isset($_SESSION['dark'])){echo"<style>body{ color: #fff; background: #000;} form{ background: #090909; box-shadow: 0px 0px 7px #777;} input{ color: #fff;} hr{ border-top: 1px solid #777;} .res{ border: 1px solid #000;} .res:hover{ border: 1px solid #777;} .res ln{ color: #ddd}</style>";}?>
<body>
    <a href="./" class="a">Ixeo</a>
    <form id="f"><input id="q" value="<?php echo $q?>" autocomplete="off" name="q" placeholder="Search Ixeo or type a URL"><button type="submit">Search</button></form>
    <div id="resultStats"></div>
    <hr>
    <script>
        const results = [<?php
            require("../../app/server/mysql/db.php");

            $conn = new mysqli($host, $user, $pass, 'Ixeo');

            $timenow = time();
            $qSecure = trim($conn->real_escape_string($q));

            function searchFunction($query){
                global $conn, $qSecure;
                $result = $conn->query("SELECT url, title, description FROM directory where ".$query." ORDER BY score ASC LIMIT 20");
                $count = $conn->query("SELECT COUNT(*) FROM directory where ".$query);
                $rowCount = mysqli_fetch_array($count);
                $count = $rowCount[0];
                if(mysqli_num_rows($result)){
                    while($row = $result->fetch_assoc()) {
                      $descriptionParsed = trim(preg_replace('/\s+/', ' ', $row["description"]));
                      echo "{u:'" . $row["url"]. "',t:`" . $row["title"] . "`,d:`" . $descriptionParsed . "`},";
                    }
                }
                return $count;
            }

            // new smid

            $connMidelight = new mysqli($host, $user, $pass, 'midelight');
            $UA = $connMidelight->real_escape_string($_SERVER['HTTP_USER_AGENT']);
            $IP = $connMidelight->real_escape_string($_SERVER['REMOTE_ADDR']);
            $smid = $connMidelight->query("SELECT id FROM mid WHERE UserAgent='$UA' AND IP='$IP'");
            if(mysqli_num_rows($smid)){
                $rowSmid = mysqli_fetch_array($smid);
                $smid = $rowSmid[0];
            }
            else{
                $smidid = $conn->query("SHOW TABLE STATUS FROM `midelight` WHERE `name` LIKE 'mid'");
                $smid = mysqli_fetch_assoc($smidid)["Auto_increment"];
                $smidNew = $connMidelight->query("INSERT INTO `mid`(`IP`, `UserAgent`, `date`) VALUES ('$IP','$UA','$timenow')");
            }

            // include session/index.php later on when accounts are introduced
            $conn->query("INSERT INTO `queries`(`mid`, `query`, `date`) VALUES ('$smid','$qSecure','$timenow')");

            if(str_starts_with(strtolower($qSecure), "https://") || str_starts_with(strtolower($qSecure), "http://")){
                header("Location: " . $qSecure);
                exit();
            }
            $count = searchFunction("title LIKE '%".$qSecure."' OR description LIKE '%".$qSecure."'");
            if($count < 20){
                if($count != 0){
                    $lastCount = $count;
                }
                $count = searchFunction("title LIKE '%".$qSecure."%' OR description LIKE '%".$qSecure."%'");
                if($count != 0){
                    $count += $lastCount;
                }
            }
            if($count == 0){
                $count = searchFunction("textContent LIKE '%".$qSecure."%'");
            }
    
            // else{
            //     $thoroughtResult = $conn->query("SELECT url, title, description FROM directory where textContent LIKE '%".$qSecure."%' ORDER BY score ASC LIMIT 20");
            //     $count = $conn->query("SELECT COUNT(*) FROM directory where textContent LIKE '%".$qSecure."%'");
            //     if(mysqli_num_rows($thoroughtResult)){
            //         while($row = $thoroughtResult->fetch_assoc()) {
            //             $descriptionParsed = trim(preg_replace('/\s+/', ' ', $row["description"]));
            //             echo "{u:'" . $row["url"]. "',t:`" . $row["title"] . "`,d:`" . $descriptionParsed . "`},";
            //         }
            //     }
            // }
        ?>];

        const count = <?php echo $count;?>;

        const resultStats = document.getElementById("resultStats");
        resultStats.innerText = count + " results found"

        const search = "<?php echo $q?>";
        const resultsDiv = document.createElement("div")
        resultsDiv.classList.add("results");
        document.body.appendChild(resultsDiv);

        for (let i = 0; i < results.length; i++) {
            const element = results[i];
            const div = document.createElement("div");
            div.innerHTML = '<a href="'+element.u+'">'+
                // '<img width="128px" src="design/16.jpg">'+
                '<ln>'+element.u+'</ln>'+
                '<h1>'+element.t+'</h1>'+
            '</a>'+
            '<p>'+element.d+'</p>';
            resultsDiv.appendChild(div);
        }
        // var calculated = eval(search)

        // var calc = document.createElement("div")
        // calc.id = 'lo'
        // calc.innerHTML = calculated
        // document.body.appendChild(calc)
        var q = document.getElementById("q")
        var f = document.getElementById("f")
        q.onfocus = function(){
            f.style.boxShadow = "0 -2px 3px 0 #65baff99, 0 2px 3px 0px #a67fff99"
        }
        q.onblur = function(){
            f.style.removeProperty("box-shadow")
        }
    </script>
</body>
</html>