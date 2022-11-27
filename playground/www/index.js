import * as hbs from "hbs-playground";

document.getElementById("render").addEventListener('click', (event) => {
    var template = document.getElementById("template").value;
    var data = document.getElementById("data").value;
    var json_data = JSON.parse(data);

    try {
        var result = hbs.render(template, json_data);
        document.getElementById("output").value = result;
    } catch (e) {
        console.log(e);
    }
});
