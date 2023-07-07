<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>WebScan v1.0</title>
    <style>
        body{background:#000;color:#fff}
    </style>
</head>
<body>
    <button onclick="start()" id="start">Start</button>
    <a>Progress:
        <?php //include "indexinternetxx.php"?>
    </a>
    <a id="result"></a>
    <script src="https://ajax.googleapis.com/ajax/libs/jquery/3.5.1/jquery.min.js"></script>
    <script>
        function start(){
            document.getElementById("start").style.display = "none";

            $.ajax({
            url: "indexinternetxx.php",
            success: function (response) {
                document.getElementById("result").innerHTML = document.getElementById("result").innerHTML + ";" + response;
            },
            error: function() {
                console.log("ERROR! Couldn't connect to indexinternet script.");
            }
            });
        }
    </script>
</body>
</html>