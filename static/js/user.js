$( document ).ready( () => {
    
    $( "#editProfile" ).click(function( event ) {

        if ($( "#profileForm" ).find(':input').prop('disabled')) {
            $( "#profileForm" ).find(':input').prop('disabled', false);
            $( "#profileForm button" ).prop('hidden', false);
            $( this ).text("abort");
        } else {
            $( "#profileForm" ).find(':input').prop('disabled', true);
            $( "#profileForm button" ).prop('hidden', true);
            $( this ).text("edit");
        }
    });

    $( "#profileForm" ).submit(function( event ) {

        // Prevent the form from submitting
        event.preventDefault();
        
        // Cache form for reuse
        let $form = $(this);
        
        // Paragraph to show result message
        const $result = $("#resultProfile");

        const $email = $("input[name='email']", this);
        
        if (!validateEmail($email.val())) {
            $result.text("Invalid E-Mail");
            $result.css("color", "red");
            return false;
        }

        $.ajax({
            // The URL fro the request
            url : $form.prop('action'),

            // The data to send
            data : $form.serialize(),

            type : $form.prop('method'),

            dataType : "json",
        }).done( (json) => {
            console.log(json);
            $( "#profileForm" ).find(':input').prop('disabled', true);
            $( "#profileForm" ).find( 'button' ).prop('hidden', true);
            $( "#editProfile" ).text("edit");

            if (json['status'] === 'ok') {
                $result.css("color", "green");
            } else {
                $result.css("color", "red");
            }
            $result.text(json['message']);
        }).fail( (xhr, status, error) => {
            $result.text("Ooops... something went wrong");
            $result.css("color", "red");
        })
    });
});

function validateEmail(email) {
  const re = /^(([^<>()[\]\\.,;:\s@\"]+(\.[^<>()[\]\\.,;:\s@\"]+)*)|(\".+\"))@((\[[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\])|(([a-zA-Z\-0-9]+\.)+[a-zA-Z]{2,}))$/;
  return re.test(email);
}
