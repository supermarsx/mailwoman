# Administrator authenticator setup

Open **My preferences → Manage authenticator and recovery codes**. This page manages the signed-in administrator's factors; it does not change a mailbox user's authentication.

Choose **Set up authenticator** and complete the secure identity confirmation. Scan the QR code with an authenticator app, or enter the displayed manual key. The QR image is generated in the browser. Enter the current six-digit code to activate the factor. Pending setup expires after ten minutes and is bound to the administrator session that started it.

If new recovery codes are issued, save them before acknowledging the screen. They are displayed only once. Recovery codes are shared by the administrator's enrolled second factors, and each can be used once. **Replace recovery codes** requires confirmation and invalidates the previous unused set.

To remove an authenticator, choose **Remove authenticator** and confirm the operation. The central identity-confirmation dialog accepts an enrolled factor or a recovery code with the password when those methods are available. It does not require a second authenticator code after that confirmation. A lost authenticator can therefore be removed using an unused recovery code. Externally managed administrator accounts display their managed-authentication status instead of local enrollment controls.

Setup keys and recovery codes are not written to appearance or dashboard preferences. Navigating away from a pending setup or an unacknowledged recovery-code screen asks before discarding the displayed material.
