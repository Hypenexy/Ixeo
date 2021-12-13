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
    <hr>
    <div class="res">
        <a href="https://websitelink.com">
            <img width="128px" src="design/16.jpg">
            <ln>https://websitelink.com</ln>
            <h1>how to make websites in html</h1>
        </a>
        <p>In this website you learn how to make cool websites!</p>
    </div>
    <script>
        var search = "<?php echo $q?>";
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