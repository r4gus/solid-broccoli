window.misc = window.misc || {};

misc.show_message = function (head, body, type, deltaT = 10000) {
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
