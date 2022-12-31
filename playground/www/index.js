import * as hbs from "hbs-playground";

document.getElementById("rust-render").addEventListener('click', (event) => {
    var template = document.getElementById("template").value;
    var data = document.getElementById("data").value;
    var json_data = JSON.parse(data);

    try {
        var result = hbs.render(template, json_data);
        document.getElementById("rust-output").value = result;
        document.getElementById("rust-error").innerText = "";
    } catch (e) {
        document.getElementById("rust-error").innerText = e.toString();
    }
});

document.getElementById("js-render").addEventListener('click', (event) => {
    var template = document.getElementById("template").value;
    var data = document.getElementById("data").value;
    var json_data = JSON.parse(data);

    try {
        var template = Handlebars.compile(template);
        var result = template(json_data);
        document.getElementById("js-output").value = result;
        document.getElementById("js-error").innerText = "";
    } catch (e) {
        document.getElementById("js-error").innerText = e.toString();
    }
});
