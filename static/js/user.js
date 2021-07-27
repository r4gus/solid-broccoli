$( document ).ready( () => {

    function show_message(head, body, type, deltaT = 10000) {
        const $msg = $(`<div class="alert alert-dismissible alert-${type}">
          <button type="button" class="btn-close" data-bs-dismiss="alert"></button>
          <strong>${head}</strong>: ${body}
        </div>`);

        $( "#resultProfile" ).prepend($msg);

        setTimeout(function() {
            $msg.fadeOut( "slow", function() {
                $msg.remove();    
            });
        }, deltaT);
    }
    
    $( "#profileForm" ).submit(function( event ) {

        // Prevent the form from submitting
        event.preventDefault();
        
        // Cache form for reuse
        let $form = $(this);
        
        const $email = $("input[name='email']", this);
        
        if (!validateEmail($email.val())) {
            show_message("Warning", "Invalid E-Mail", "warning");        
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
            if (json['status'] === 'ok') {
                show_message("Success", json['message'], "success");        
            } else {
                show_message("Warning", json['message'], "warning");        
            }
        }).fail( (xhr, status, error) => {
            show_message("Error", "Unable to process request", "danger");        
        })
    });

    $( "#passwordForm").submit(function( event ) {
        // Prevent the form from submitting
        event.preventDefault();
        
        // Cache form for reuse
        let $form = $(this);

        $.ajax({
            // The URL fro the request
            url : $form.prop('action'),

            // The data to send
            data : $form.serialize(),

            type : $form.prop('method'),

            dataType : "json",
        }).done( (json) => {
            if (json['status'] === 'ok') {
                show_message("Success", json['message'], "success");        
            } else {
                show_message("Warning", json['message'], "warning");        
            }
            $result.text(json['message']);
        }).fail( (xhr, status, error) => {
            show_message("Error", "Unable to process request", "danger");        
        })
    });

    $( "#delteConfirmation" ).on("input", function() {
        const val = $( this ).val();
        const expected = $( this ).prop('placeholder');
        const $button = $( "#deleteUserBtn" );

        console.log(val + " == " + expected);
        
        if (val === expected) {
            $button.prop('disabled', false);
        } else {
            $button.prop('disabled', true);
        }

        console.log($button.prop('disabled'));
    });

});

function validateEmail(email) {
  const re = /^(([^<>()[\]\\.,;:\s@\"]+(\.[^<>()[\]\\.,;:\s@\"]+)*)|(\".+\"))@((\[[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\])|(([a-zA-Z\-0-9]+\.)+[a-zA-Z]{2,}))$/;
  return re.test(email);
}
