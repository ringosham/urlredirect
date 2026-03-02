package main

import (
	"flag"
	"fmt"
	"log"
	"net"
	"net/http"
	"os"
	"os/exec"
	"strings"
)

func getBridgeIP() (string, error) {
	var bridges = make([]string, 0)
	libvirtEnv := os.Getenv("LIBVIRT")
	dockerEnv := os.Getenv("DOCKER")

	if dockerEnv == "1" {
		log.Println("Using Docker bridge (DOCKER=1)")
		bridges = append(bridges, "docker0", "br0")
	} else if libvirtEnv == "1" {
		log.Println("Using libvirt bridge (LIBVIRT=1)")
		bridges = append(bridges, "virbr0")
	} else {
		// Default to libvirt if neither are defined
		log.Println("Using libvirt bridge (default)")
		bridges = []string{"virbr0"}
	}

	for _, bridge := range bridges {
		iface, err := net.InterfaceByName(bridge)
		if err != nil {
			continue
		}

		addrs, err := iface.Addrs()
		if err != nil {
			continue
		}

		for _, addr := range addrs {
			if ipnet, ok := addr.(*net.IPNet); ok {
				// Return IPv4 address
				if ipv4 := ipnet.IP.To4(); ipv4 != nil {
					log.Printf("Found bridge IP: %s on %s\n", ipv4.String(), bridge)
					return ipv4.String(), nil
				}
			}
		}
	}

	return "", fmt.Errorf("no bridge IP found (LIBVIRT=%s, DOCKER=%s)", libvirtEnv, dockerEnv)
}

// handleRedirect handles incoming HTTP requests and opens the URL
func handleRedirect(w http.ResponseWriter, r *http.Request) {
	// Get the target URL from query parameter
	targetURL := r.URL.Query().Get("l")
	if targetURL == "" {
		http.Error(w, "Missing 'l' query parameter", http.StatusBadRequest)
		return
	}

	// Ensure http/https
	if !strings.HasPrefix(targetURL, "http://") && !strings.HasPrefix(targetURL, "https://") {
		targetURL = "https://" + targetURL
	}

	log.Printf("Opening URL: %s\n", targetURL)

	cmd := exec.Command("xdg-open", targetURL)
	if err := cmd.Start(); err != nil {
		log.Printf("Error opening URL: %v\n", err)
		http.Error(w, fmt.Sprintf("Failed to open URL: %v", err), http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "text/plain")
	if _, err := fmt.Fprintf(w, "Opening URL: %s\n", targetURL); err != nil {
		log.Printf("Error writing response: %v\n", err)
	}
}

func main() {
	portFlag := flag.String("p", "10080", "Port to listen on")
	flag.Parse()

	bridgeIP, err := getBridgeIP()
	if err != nil {
		log.Fatalf("Failed to find bridge IP: %v\n", err)
	}

	listenAddr := bridgeIP + ":" + *portFlag
	log.Printf("Starting URL redirect server on %s\n", listenAddr)

	http.HandleFunc("/", handleRedirect)
	if err := http.ListenAndServe(listenAddr, nil); err != nil {
		log.Fatalf("Server failed: %v\n", err)
	}
}
