$( document ).ready( () => {
    
    $( "#rmForm" ).submit(function(event) {
        event.preventDefault();

        let $form = $(this);

        $.ajax({
            url: $form.prop('action'),

            data: $form.serialize(),

            type: $form.prop('method'),

            dataType: "json",
        }).done( (json) => {
            if (json['status'] === 'ok') {
                misc.show_message("Success", json['message'], "success");        
            } else {
                misc.show_message("Error", json['message'], "danger");        
            }
        }).fail( (xhr, status, error) => {
            misc.show_message("Error", error, "danger");        
        })
    });

    
});

function getRm(uid) {
    $.ajax({
        url: `/api/exercise/rm/${uid}`,
        type: 'GET',
    }).done( (json) => {
        if (json['status'] && json['status'] === 'error') {
            misc.show_message("Error", json['message'], "danger");        
        } else {
            buildTable(json);
        }
    }).fail( (xhr, status, error) => {
        console.log(error);
    });
}

function buildTable(json) {
    let reps = new Set();
    let exercises = new Set();
    let reps_ex_to_ex = new Map();
    console.log(json);

    for (let [ex, value1] of Object.entries(json)) {
        for (let [rep, value2] of Object.entries(value1)) {
            let k = `${rep}-${ex}`;
            let last_exercise = value2[0];
            reps_ex_to_ex.set(k, last_exercise);
            reps.add(rep);
            exercises.add(ex);
        }
    }

    reps = [...Array.from(reps).sort( (a, b) => Number(a) > Number(b) )];
    exercises = [...Array.from(exercises).sort( (a, b) => a > b )];

    $( "#rmTableWrapper" ).empty();

    var html = '<table class="table table-striped" id="rmTable"><thead id="rmReps"><tr>';
    html += '<th scope="col">Exercise</th>';
    reps.forEach( (val, idx, arr) => html+= `<th scope="col">${val} RM</th>` );
    html += '</thead><tbody id="rmBody">';
    exercises.forEach( (val1, idx1, arr1) => {
        html += `<tr><th scope="row">${val1}</th>`;
        reps.forEach( (val2, idx2, arr2) => {
            let idx = `${val2}-${val1}`;
            let v;
            if (reps_ex_to_ex.has(idx)) {
                let obj = reps_ex_to_ex.get(idx);
                v = `${obj.weight} ${obj.unit}`;
            } else {
                v = "";
            }
            html += `<td>${v}</td>`;
        });
        html += '</tr>'; 
    });
    html += '</tbody></table>';
    $( html ).appendTo( "#rmTableWrapper" );
}
