package com.github.apohrebniak.hoverui.ui;

import com.github.apohrebniak.hoverui.KvNodeService;
import com.github.apohrebniak.hoverui.domain.KvEntity;
import com.github.apohrebniak.hoverui.domain.MemberEntity;
import com.vaadin.flow.component.button.Button;
import com.vaadin.flow.component.formlayout.FormLayout;
import com.vaadin.flow.component.grid.Grid;
import com.vaadin.flow.component.html.Label;
import com.vaadin.flow.component.icon.Icon;
import com.vaadin.flow.component.icon.VaadinIcon;
import com.vaadin.flow.component.orderedlayout.VerticalLayout;
import com.vaadin.flow.component.tabs.Tab;
import com.vaadin.flow.component.tabs.Tabs;
import com.vaadin.flow.component.textfield.TextField;
import com.vaadin.flow.data.binder.Binder;
import com.vaadin.flow.data.provider.DataProvider;
import com.vaadin.flow.data.provider.ListDataProvider;
import com.vaadin.flow.data.value.ValueChangeMode;
import com.vaadin.flow.router.BeforeEvent;
import com.vaadin.flow.router.HasUrlParameter;
import com.vaadin.flow.router.Route;
import com.vaadin.flow.spring.annotation.UIScope;
import org.springframework.beans.factory.annotation.Autowired;

import java.util.*;

@Route("node")
@UIScope
public class NodeView extends VerticalLayout implements HasUrlParameter<String> {

  private static final String TAB_MEMBERS = "Members";
  private static final String TAB_KV = "Key-Values";

  //internal state
  private KvNodeService nodeService;
  private Map<Tab, TabPage> tabToPage = new HashMap<>();
  private ListDataProvider<KvEntity> kvDataProvider;
  private List<KvEntity> kvData = new ArrayList<>();

  //params
  private String host;
  private Integer port;

  //views
  private Label title;
  private Tabs tabs;
  private TabPage membersPage;
  private TabPage kvPage;
  private Tab tabMembers;
  private Tab tabKv;
  private FormLayout editPanel;
  private KvEntity editableEntity;
  private TextField keyField;
  private TextField valueField;
  private Button addButton;
  private Button removeButton;

  public NodeView(@Autowired KvNodeService nodeService) {
    this.nodeService = nodeService;
    this.kvDataProvider = DataProvider.ofCollection(kvData);
    this.title = new Label();
    this.tabs = new Tabs();
    this.editPanel = new FormLayout();
    this.editableEntity = KvEntity.builder().build();
    this.keyField = new TextField();
    this.valueField = new TextField();
    this.addButton = new Button("Add");
    this.removeButton = new Button("Remove");
  }

  private void setupUi() {
    /*Title*/
    title.setText(String.format("KV node %s:%d", this.host, this.port));
    title.getStyle().set("font-size", "40px");
    title.getStyle().set("font-weight", "bold");
    title.getStyle().set("font-family", "monospace");
    add(title);

    /*Tabs*/
    tabs.setSizeFull();
    add(tabs);

    /*Tab pages*/
    membersPage = setupMembersPage();
    kvPage = setupKvPage();
    add(membersPage);
    add(kvPage);

    tabMembers = new Tab(TAB_MEMBERS);
    tabKv = new Tab(TAB_KV);
    tabToPage.put(tabMembers, membersPage);
    tabToPage.put(tabKv, kvPage);
    tabs.add(tabMembers);
    tabs.add(tabKv);
    tabs.setFlexGrowForEnclosedTabs(1);

    tabs.addSelectedChangeListener(event -> {
      Tab selectedTab = event.getSource().getSelectedTab();

      tabToPage.entrySet().stream()
          .forEach(e -> e.getValue().setVisible(false));

      tabToPage.get(selectedTab).setVisible(true);
    });

    tabs.setSelectedTab(tabMembers);
    membersPage.setVisible(true);
  }

  @Override
  public void setParameter(BeforeEvent event, String parameter) {
    String[] args = parameter.split("_");
    if (args.length == 2) {
      this.host = args[0];
      this.port = Integer.valueOf(args[1]);
    }

    setupUi();
  }

  private TabPage setupMembersPage() {
    TabPage page = new TabPage();
    page.setVisible(false);

    List<MemberEntity> members = nodeService
        .getMembers(host, port);

    if (members != null) { //TODO react with error
      members.stream()
          .map(MemberListItem::from)
          .forEach(page::add);
    }

    return page;
  }

  private TabPage setupKvPage() {
    TabPage page = new TabPage();
    page.setVisible(false);

    Grid<KvEntity> grid = new Grid<>(KvEntity.class);
    grid.setDataProvider(kvDataProvider);
    grid.setSelectionMode(Grid.SelectionMode.SINGLE);

    Optional<List<KvEntity>> kvs =
        nodeService.getKvAll(host, port);
    kvs.ifPresent(l -> {
      kvData.addAll(l);
      kvDataProvider.refreshAll();
    });//TODO react with error


    Binder<KvEntity> binder = new Binder<>();

    keyField.setLabel("Key");
    keyField.setValueChangeMode(ValueChangeMode.EAGER);
    keyField.setPlaceholder("foo");
    Binder.Binding<KvEntity, String> keyBinding = binder.forField(keyField)
        .withValidator(value -> !value.trim().isBlank() && value.matches("^([A-Z]|[a-z])+$"),
            "Should be a non empty sequence of letters!")
        .bind(KvEntity::getKey, KvEntity::setKey);
    keyField.addValueChangeListener(event -> keyBinding.validate());

    valueField.setLabel("Value");
    valueField.setPlaceholder("bar");
    keyField.setValueChangeMode(ValueChangeMode.EAGER);
    Binder.Binding<KvEntity, String> valueBinding = binder.forField(valueField)
        .withValidator(value -> !value.trim().isBlank(),
            "Cannot be empty")
        .bind(KvEntity::getValue, KvEntity::setValue);
    valueField.addValueChangeListener(event -> valueBinding.validate());

    addButton.addClickListener(event -> {
      if (binder.writeBeanIfValid(editableEntity)) {
        KvEntity copy = KvEntity.builder()
            .key(editableEntity.getKey())
            .value(editableEntity.getValue())
            .build();
        if (nodeService.addKv(host, port, copy)) {
          kvData.removeIf(item -> item.getKey().equals(copy.getKey()));
          kvData.add(copy);
          kvDataProvider.refreshAll();
        }
        editableEntity.setKey(null);
        editableEntity.setValue(null);
      }
    });

    removeButton.setEnabled(false);
    removeButton.setIcon(new Icon(VaadinIcon.TRASH));
    removeButton.addClickListener(event -> {
      if (event.getButton() != 0) return;

      Set<KvEntity> selectedItems = grid.getSelectedItems();
      if (!selectedItems.isEmpty()) {
        selectedItems.forEach(item -> {
          if (nodeService.deleteKv(host, port, item)) {
            kvData.remove(item);
          }
        });
      }
      kvDataProvider.refreshAll();
      removeButton.setEnabled(false);
    });

    grid.addSelectionListener(event -> {
      if (event.getAllSelectedItems().isEmpty()) {
        removeButton.setEnabled(false);
      } else {
        removeButton.setEnabled(true);
      }
    });

    editPanel.add(keyField, valueField, addButton, removeButton);
    editPanel.setResponsiveSteps(
        new FormLayout.ResponsiveStep("0", 1),
        new FormLayout.ResponsiveStep("21em", 2),
        new FormLayout.ResponsiveStep("21em", 3),
        new FormLayout.ResponsiveStep("21em", 4));


    page.add(editPanel);
    page.add(grid);

    return page;
  }
}
