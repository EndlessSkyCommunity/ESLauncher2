# ESLauncher2
A program for organizing Endless Sky installations.

### Currently supported features:
- Install [Continuous Builds](https://github.com/endless-sky/endless-sky/releases/tag/continuous)
- Install Development build of PRs
- Install specific versions
- Update instances
- Play instances
- Install & manage plug-ins

### Requirements
ESLauncher2 requires DirectX 11 or 12, Vulkan or Metal. See also [#4](https://github.com/EndlessSkyCommunity/ESLauncher2/issues/4).


### Installation instructions for Mac
#### Additional requirements for Mac ####
**Important**: currently, the launcher works only when run from a console which has access to the full filesystem.
We are working on a solution to overcome this.

#### Instructions ####
- Download the zip to a location of your choice
- Open a Terminal session and navigate to the zip file
- Unzip the launcher (```unzip eslauncher2-x86_64-apple-darwin.zip```)
- mark it as executable (```chmod 755 eslauncher2-x86_64-apple-darwin```)
- run the launcher (```./eslauncher2-x86_64-apple-darwin```)

#### Full Disk Access for the Console ####
- Open *System Preferences*
- Navigate to *Security & Privacy*
- Select the *Privacy* tab
- Scroll down to the item *Full Disk Access*
- If *Terminal* is not present or not ticked, add it or activate the tick box
- To add Terminal:
¨* Unlock the settings by clicking on the lock symbol
¨* If *Terminal* is not there, click on the plus
¨* Navigate to *Applications* -> *Utilities* -> *Terminal*
¨* Select it and click *Open*
¨* It should now appear in the list, make sure the box before it is ticked
