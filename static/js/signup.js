$( document ).ready( () => {

   // Check if both passwords match before submiting the form.
   $( "#signupForm" ).submit(function (event) {
        const pw1 = $("#floatingPassword1").val();
        const pw2 = $("#floatingPassword2").val();

        if (pw1 !== pw2) {
            // Prevent the form from submitting
            event.preventDefault();

            // TODO: additional error handling
        }
   });

    $( "#username" ).on("input", function() {
        const val = $( this ).val();
        const url = "/username/" + val;
        
        if (val.length > 0) {
            $.ajax({
                // The URL fro the request
                url : url,

                type : "GET",

                dataType : "json",
            }).done( (json) => {
                if (json['status'] === 'ok') {
                    $( this ).removeClass("is-invalid");
                    $( this ).addClass("is-valid");
                    $( "#usernameFeedback" ).removeClass("invalid-feedback");
                    $( "#usernameFeedback" ).text("");
                } else {
                    $( this ).removeClass("is-valid");
                    $( this ).addClass("is-invalid");
                    $( "#usernameFeedback" ).addClass("invalid-feedback");
                    $( "#usernameFeedback" ).text(json['message']);
                }
            })
        } else {
            $( this ).removeClass("is-valid");
            $( this ).removeClass("is-invalid");
        }
    });
});
