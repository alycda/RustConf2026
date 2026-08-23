{ pkgs, ... }:
{
  imports = [ ../home.nix ];

  # Kotlin track: kotlinc + a JDK on PATH, so `just check` goes green without
  # sdkman. JAVA_HOME points at the JDK because kotlinc and java want it, not
  # because anything here needs jni.h — nothing in this repo uses JNI.
  #
  # jna is here because the golden day's Kotlin harness goes through JNA, not
  # JNI: UniFFI's Kotlin backend calls the cdylib through it, so jna.jar has
  # to be on the classpath at compile time and at run time. Without it,
  # `days/2024-12-03/uniffi/build-and-test.sh kotlin` stops before it starts.
  # (Substitutes from cache on aarch64-linux; nothing compiles.)
  home.packages = with pkgs; [ kotlin jdk17 jna ];
  home.sessionVariables.JAVA_HOME = "${pkgs.jdk17.home}";
  home.sessionVariables.JNA_JAR = "${pkgs.jna}/share/java/jna.jar";
}
