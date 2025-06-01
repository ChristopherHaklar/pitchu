# Pitchu

### *Warning:* This is a vibecoded project, written by Gemini 2.5 Flash and debugged and tweaked by hand.

Pitchu (Pit-chu) Rust program is designed to allow you to control a GBA emulator by singing. It process real-time microphone input, identifies the vocal pitch, and then translates specific pitches into keystrokes.

## Getting Started

### Prerequisites

* Rust

### Program Execution

1. Permission Granting (macOS Specific \- CRITICAL\!):  
   On macOS, explicit system permissions are indispensable for the program's operation, encompassing both microphone access and keyboard input simulation.  
   * **Microphone Access:**  
     * Go to System Settings
     * Go to Privacy & Security.  
     * Select Microphone.  
     * Confirm that the terminal application in use (e.g., Terminal.app, iTerm2) or the compiled executable of the Rust program (if executed directly) is listed and enabled. If not present, executing the program once should prompt macOS to request this permission.  
   * **Accessibility Access (for keyboard simulation):**  
     * Navigate to System Settings (or System Preferences).  
     * Proceed to Privacy & Security.  
     * Select Accessibility.  
     * The terminal application or the compiled executable must be added to the list on the right and granted a checkmark. This authorization is necessary for the program to simulate keyboard events effectively.  
2. Program Initiation:  
   From the terminal, within the project directory, execute the program using the following command:  
   RUST\_LOG=info cargo run

   * RUST\_LOG=info: This setting enables the display of essential information and error messages.  
   * RUST\_LOG=debug: This setting provides more detailed debugging output, including instances where pitches are detected but not mapped, or when throttling mechanisms are engaged. This level of verbosity is beneficial for fine-tuning operations.

Upon successful initiation, console output confirming the activation of the audio stream should be observed.

3. Target Application Focus:  
   It is imperative to ensure that mGBA (or any other intended target application) is the active window on the desktop prior to interaction.  
4. Vocalization:  
   Users should vocalize into the microphone, endeavoring to sustain notes consistently within the frequency ranges defined in src/main.rs.

## **Customization**

### **Adjustment of Pitch-to-Key Mappings**

The core functionality of the program is governed by the `map_frequency_to_key` function, located within `src/main.rs`.

```
fn map\_frequency\_to\_key(freq: f32) \-\> Option\<Key\> {  
    match freq {  
        // Example:  
        100.0..=115.0 \=\> Some(Key::DownArrow),   // D-Pad Down  
        115.1..=130.0 \=\> Some(Key::LeftArrow),    // D-Pad Left  
        // ... (additional mappings) ...  
        \_ \=\> None, // No key mapped for other frequencies  
    }  
}
```

**Customization Procedure:**

1. **Vocal Range Determination:**  
   * Utilize an online vocal tuner or a smartphone-based frequency analyzer application.  
   * Vocalize various notes that are comfortable and can be sustained with ease. Record the stable frequency readings corresponding to each note.  
   * The objective is to define ranges that are sufficiently distinct to allow for reliable vocal differentiation.  
2. **Modification of the match Statement:**  
   * Adjust the f32 ranges (e.g., 100.0..=115.0) to align with the empirically observed frequencies.  
   * Alter the Key values (e.g., Key::DownArrow, Key::Layout('x')) to correspond with the desired keyboard inputs for the target application. Reference to the enigo documentation is advised for a comprehensive list of available Key variants.

### **Tuning of Pitch Detection Sensitivity**

Within the main function, the `detector.get_pitch` call is located:
```
if let Some(pitch) \= detector.get\_pitch(\&audio\_window, sample\_rate, 0.7, 0.2) {  
                                                                 // ^^^^^^  
                                                                 // Clarity Threshold
```
* **Clarity Threshold (e.g., 0.7):** This parameter, ranging from 0.0 to 1.0, dictates the requisite "clarity" of a detected pitch for it to be considered valid.  
  * **Higher values (e.g., 0.9):** A stricter criterion, resulting in fewer false positives but potentially overlooking some valid pitches if microphone quality or vocalization is imperfect.  
  * **Lower values (e.g., 0.5):** A more lenient criterion, leading to the detection of a broader range of pitches but with an increased susceptibility to background noise or less stable vocalizations.  
  * Adjustment of this value should be performed based on the ambient environment and the quality of the microphone.

Contributions and issues are welcome.