;;; pipemacs.el --- Helpers to pipe data into emacs -*- lexical-binding: t -*-
;;; Commentary:
;;;
;;; This library includes helper code to pipe data from standard input into Emacs.
;;; Note that all of the definitions are wrapped in a progn form so that this content
;;; can be passed to emacs via the --eval flag.

;;; Code:

(progn
  (defun pipemacs--process-filter (buffer input)
    "Read INPUT from the given process and append it to BUFFER."

    (with-current-buffer buffer
      (save-excursion
        (goto-char (point-max))
        (insert input))))

  (defun pipemacs-read-data-into-buffer (port mode buffer-name writeback)
    "Read data from the given PORT on localhost into BUFFER-NAME.

Enable the major mode MODE in the new buffer.

If WRITEBACK is t, write the contents of the buffer back to the
server when emacs is killed.

Note that input is sent line-by-line, so each chunk is guaranteed
to be valid utf-8."
    (let* ((target-buffer (get-buffer-create buffer-name))
           (network-process (make-network-process :name "pipemacs-network-proc"
                                                   :host "127.0.0.1"
                                                   :filter (lambda (_proc input) (pipemacs--process-filter target-buffer input))
                                                   :service port)))
      (switch-to-buffer target-buffer)
      (funcall (intern mode))
      (when writeback
        (add-hook 'kill-buffer-hook (lambda ()
                                      (with-current-buffer target-buffer
                                        (process-send-string network-process (buffer-string))
                                        (delete-process network-process))))))))

;;; pipemacs.el ends here
