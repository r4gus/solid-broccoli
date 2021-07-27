$( document ).ready( () => {


    function check_input() {
        if ($( ".is-invalid" ).length > 0) {
            $( "#submitBtn" ).prop("disabled", true);
        } else {
            $( "#submitBtn" ).prop("disabled", false);
        }
    }

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
                check_input();
            })
        } else {
            $( this ).removeClass("is-valid");
            $( this ).removeClass("is-invalid");
        }

    });

    $( "#email" ).on("input", function() {
        const val = $( this ).val();
        const url = "/email/" + val;
        
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
                    $( "#emailFeedback" ).removeClass("invalid-feedback");
                    $( "#emailFeedback" ).text("");
                } else {
                    $( this ).removeClass("is-valid");
                    $( this ).addClass("is-invalid");
                    $( "#emailFeedback" ).addClass("invalid-feedback");
                    $( "#emailFeedback" ).text(json['message']);
                }
                check_input();
            })
        } else {
            $( this ).removeClass("is-valid");
            $( this ).removeClass("is-invalid");
        }

    });

    $( "#password1" ).on("input", check_pw);
    $( "#password2" ).on("input", check_pw);

    function check_pw() {
        const pw1 = $( "#password1" ).val();
        const pw2 = $( "#password2" ).val();

        $.ajax({
                // The URL fro the request
                url : "/password",

                type : "POST",

                data:  "pw1="+pw1+"&pw2="+pw2,

                dataType : "json",
            }).done( (json) => {
                if (json['status'] === 'ok') {
                    $( "#password1" ).removeClass("is-invalid");
                    $( "#password1" ).addClass("is-valid");
                    $( "#password2" ).removeClass("is-invalid");
                    $( "#password2" ).addClass("is-valid");
                    $( "#passwordFeedback" ).removeClass("invalid-feedback");
                    $( "#passwordFeedback" ).text("");
                } else {
                    $( "#password1" ).removeClass("is-valid");
                    $( "#password1" ).addClass("is-invalid");
                    $( "#password2" ).removeClass("is-valid");
                    $( "#password2" ).addClass("is-invalid");
                    $( "#passwordFeedback" ).addClass("invalid-feedback");
                    $( "#passwordFeedback" ).text(json['message']);
                }

                check_input();
            })

    }
});
