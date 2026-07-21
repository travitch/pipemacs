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

  (defun pipemacs-read-data-into-buffer (port mode buffer-name)
    "Read data from the given PORT on localhost into BUFFER-NAME.

Enable the major mode MODE in the new buffer.

Note that input is sent line-by-line, so each chunk is guaranteed
to be valid utf-8."
    (let* ((target-buffer (get-buffer-create buffer-name))
           (_network-process (make-network-process :name "pipemacs-network-proc"
                                                   :host "127.0.0.1"
                                                   :filter (lambda (_proc input) (pipemacs--process-filter target-buffer input))
                                                   :service port)))
      (switch-to-buffer target-buffer)
      (funcall (intern mode)))))

;;; pipemacs.el ends here
